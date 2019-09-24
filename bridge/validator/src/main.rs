use dotenv::dotenv;
use env_logger;
use ethabi::ParamType;
use futures::{
    future::{lazy, poll_fn},
    stream::Stream,
};
use log;
use tokio_threadpool::blocking;
use web3::{
    contract::{tokens::Tokenizable, Contract, Options},
    futures::Future,
    types::FilterBuilder,
    types::{Address, H160, H256, U256},
};

use node_runtime::{bridge, AccountId, Event};
use parity_codec::Decode;
use primitives::{crypto::Pair, sr25519};
use substrate_api_client::{hexstr_to_vec, Api};
use system;

use std::env;
use std::sync::{mpsc, Arc};
use std::thread;

mod extrinsics;

fn main() {
    env_logger::init();
    dotenv().ok();

    let (
        eth_api_url,
        eth_validator_address,
        eth_contract_address,
        eth_relay_message_hash,
        eth_approved_relay_message_hash,
        sub_api_url,
        sub_validator_mnemonic_phrase,
    ) = read_env();

    log::info!("[ethereum] api url: {:?}", eth_api_url);
    log::info!("[ethereum] contract address: {:?}", eth_contract_address);
    log::info!(
        "[ethereum] hash of RelayMessage: {:?}",
        eth_relay_message_hash
    );
    log::info!(
        "[ethereum] hash of ApprovedRelayMessage: {:?}",
        eth_approved_relay_message_hash
    );
    log::info!("[substrate] api url: {:?}", sub_api_url);

    let (_event_subscriber, _event_handler) = start_substrate_event_handler(
        sub_api_url.clone(),
        sub_validator_mnemonic_phrase.clone(),
        eth_api_url.clone(),
        eth_validator_address,
        eth_contract_address,
    );

    let mut sub_api = Api::new(sub_api_url);
    sub_api.init();
    let sub_api = Arc::new(sub_api);

    let (_eloop, transport) = web3::transports::WebSocket::new(&eth_api_url).unwrap();
    let web3 = web3::Web3::new(transport);

    let contract = Contract::from_json(
        web3.eth(),
        eth_contract_address,
        include_bytes!("../res/EthContract.abi"),
    )
    .expect("can not create contract");

    let filter = FilterBuilder::default()
        .address(vec![contract.address()])
        .topics(
            Some(vec![
                eth_relay_message_hash,
                eth_approved_relay_message_hash,
            ]),
            None,
            None,
            None,
        )
        .build();

    let fut = web3
        .eth_subscribe()
        .subscribe_logs(filter)
        .then(move |sub| {
            sub.unwrap().for_each(move |log| {
                log::info!("[ethereum] got log: {:?}", log);
                let received_relay_message = log.topics.iter().any(|addr| addr == &eth_relay_message_hash);
                let received_approved_relay_message = log.topics.iter().any(|addr| addr == &eth_approved_relay_message_hash);

                match (received_relay_message, received_approved_relay_message) {
                    (true, _) => {
                        let result = ethabi::decode(&[ParamType::FixedBytes(32), ParamType::Address, ParamType::FixedBytes(32), ParamType::Uint(256)], &log.data.0);
                        if let Ok(params) = result {
                            log::info!("[ethereum] got decoded log.data: {:?}", params);
                            if params.len() >= 4 {
                                let args = (params[0].clone(), params[1].clone(), params[2].clone(), params[3].clone());
                                let f = contract.call("approveTransfer", args, eth_validator_address, Options::default())
                                    .then(move |tx_res| {
                                        log::info!("[ethereum] called approveTransfer({:?}, {:?}, {:?}, {:?}), result: {:?}",
                                                   params[0], params[1], params[2], params[3], tx_res);
                                        Ok(())
                                    });
                                tokio::spawn(f);
                            }
                        }
                        Ok(())
                    },
                    (_, true) => {
                        let result = ethabi::decode(&[ParamType::FixedBytes(32), ParamType::Address, ParamType::FixedBytes(32), ParamType::Uint(256)], &log.data.0);
                        if let Ok(params) = result {
                            log::info!("[ethereum] got decoded log.data: {:?}", params);
                            if params.len() >= 4 {
                                let message_id = params[0].clone().to_fixed_bytes().expect("can not parse message_id");
                                let from = params[1].clone().to_address().map(|x| x.as_bytes().to_vec()).expect("can not parse 'from' address");
                                let to = params[2].clone().to_fixed_bytes().map(|x| sr25519::Public::from_slice(&x)).expect("can not parse 'to' address");
                                let amount = params[3].clone().to_uint().map(|x| x.low_u128()).expect("can not parse amount");

                                let sub_validator_mnemonic_phrase2 = sub_validator_mnemonic_phrase.clone();
                                let sub_api2 = sub_api.clone();
                                tokio::spawn(lazy(move || {
                                    poll_fn(move || {
                                        blocking(|| {
                                            mint(sub_api2.clone(), sub_validator_mnemonic_phrase2.clone(), message_id.clone(), from.clone(), to.clone(), amount);
                                            log::info!("[substrate] called eth2substrate({:?}, {:?}, {:?}, {:?})", message_id, from, to, amount);
                                        }).map_err(|_| panic!("the threadpool shut down"))
                                    })
                                }));
                            }
                        }
                        Ok(())
                    }
                    (_, _) => {
                        log::warn!("received unknown log: {:?}", log);
                        Ok(())
                    }
                }
            })
        })
        .map_err(|_| ());

    tokio::run(fut);
}

fn read_env() -> (String, Address, Address, H256, H256, String, String) {
    let eth_api_url = env::var("ETH_API_URL").expect("can not read ETH_API_URL");
    let eth_validator_address =
        env::var("ETH_VALIDATOR_ADDRESS").expect("can not read ETH_VALIDATOR_ADDRESS");
    let eth_contract_address =
        env::var("ETH_CONTRACT_ADDRESS").expect("can not read ETH_CONTRACT_ADDRESS");
    let eth_relay_message_hash =
        env::var("ETH_RELAY_MESSAGE_HASH").expect("can not read ETH_RELAY_MESSAGE_HASH");
    let eth_approved_relay_message_hash = env::var("ETH_APPROVED_RELAY_MESSAGE_HASH")
        .expect("can not read ETH_APPROVED_RELAY_MESSAGE_HASH");

    let sub_api_url = env::var("SUB_API_URL").expect("can not read SUB_API_URL");
    let sub_validator_mnemonic_phrase = env::var("SUB_VALIDATOR_MNEMONIC_PHRASE")
        .expect("can not read SUB_VALIDATOR_MNEMONIC_PHRASE");
    let _ = sr25519::Pair::from_phrase(&sub_validator_mnemonic_phrase, None)
        .expect("invalid SUB_VALIDATOR_MNEMONIC_PHRASE");

    (
        eth_api_url.to_string(),
        eth_validator_address[2..]
            .parse()
            .expect("can not parse validator address"),
        eth_contract_address[2..]
            .parse()
            .expect("can not parse contract address"),
        eth_relay_message_hash[2..]
            .parse()
            .expect("can not parse event hash"),
        eth_approved_relay_message_hash[2..]
            .parse()
            .expect("can not parse event hash"),
        sub_api_url.to_string(),
        sub_validator_mnemonic_phrase,
    )
}

fn start_substrate_event_handler(
    api_url: String,
    signer_mnemonic_phrase: String,
    eth_api_url: String,
    eth_validator_address: H160,
    eth_contract_address: Address,
) -> (thread::JoinHandle<()>, thread::JoinHandle<()>) {
    let (events_in, events_out) = mpsc::channel();

    let event_subscriber = start_event_subscriber(api_url.clone(), events_in);
    let event_handler = start_event_handler(
        api_url,
        signer_mnemonic_phrase,
        eth_api_url,
        eth_validator_address,
        eth_contract_address,
        events_out,
    );

    (event_subscriber, event_handler)
}

fn start_event_subscriber(
    api_url: String,
    events_in: mpsc::Sender<String>,
) -> thread::JoinHandle<()> {
    let mut sub_api = Api::new(api_url);
    sub_api.init();

    thread::Builder::new()
        .name("event_subscriber".to_string())
        .spawn(move || {
            sub_api.subscribe_events(events_in.clone());
        })
        .expect("can not start event_subscriber")
}

fn start_event_handler(
    api_url: String,
    signer_mnemonic_phrase: String,
    eth_api_url: String,
    eth_validator_address: H160,
    eth_contract_address: Address,
    events_out: mpsc::Receiver<String>,
) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("event_handler".to_string())
        .spawn(move || {
            let mut sub_api = Api::new(api_url);
            sub_api.init();

            let (_eloop, transport) = web3::transports::WebSocket::new(&eth_api_url).unwrap();
            let web3 = web3::Web3::new(transport);

            let contract = Contract::from_json(
                web3.eth(),
                eth_contract_address,
                include_bytes!("../res/EthContract.abi")
            ).expect("can not create contract");
            let contract = Arc::new(contract);

            for event in events_out {
                log::debug!("[substrate] got event: {:?}", event);

                let unhex = hexstr_to_vec(event);
                let mut er_enc = unhex.as_slice();
                let events = Vec::<system::EventRecord::<Event>>::decode(&mut er_enc);

                match events {
                    Some(evts) => {
                        for evr in &evts {
                            log::debug!("[substrate] decoded: phase {:?} event {:?}", evr.phase, evr.event);
                            match &evr.event {
                                Event::bridge(br) => {
                                    log::info!("[substrate] bridge event: {:?}", br);
                                    match &br {
                                        bridge::RawEvent::Transfer(message_id, from, to, amount) => {
                                            sign(&sub_api, signer_mnemonic_phrase.clone(), message_id.clone(), from.clone(), to.clone(), *amount);
                                            log::info!("[substrate] called sign({:?}, {:?}, {:?}, {:?})", message_id, from, to, amount);
                                        },
                                        bridge::RawEvent::Burned(_last_validator, _message_id, from, to, amount) => {
                                            let args = (
                                                H256::from_slice(from.as_slice()).as_fixed_bytes().into_token(),
                                                Address::from_slice(&to).into_token(),
                                                U256::from(*amount).into_token()
                                            );
                                            let contract = contract.clone();

                                            let fut = contract.call("unlock", args.clone(), eth_validator_address, Options::default())
                                                .then(move |result| {
                                                    log::info!("[ethereum] called unlock({:?}, {:?}, {:?}), result: {:?}", args.0, args.1, args.2, result);
                                                    Ok(())
                                                });
                                            tokio::run(fut);
                                        },
                                        bridge::RawEvent::Minted(message_id) => {
                                            let args = (
                                                H256::from_slice(message_id.as_slice()).as_fixed_bytes().into_token(),
                                            );
                                            let contract = contract.clone();

                                            let fut = contract.call("confirmTransfer", args.clone(), eth_validator_address, Options::default())
                                                .then(move |result| {
                                                    log::info!("[ethereum] called confirmTransfer({:?}), result: {:?}", args.0, result);
                                                    Ok(())
                                                });
                                            tokio::run(fut);

                                        },
                                        _ => {
                                            log::debug!("[substrate] ignoring unsupported balances event");
                                        },
                                    }
                                },
                                _ => {
                                    log::debug!("[substrate] ignoring unsupported module event: {:?}", evr.event)
                                    },
                            }
                        }
                    }
                    None => log::error!("[substrate] could not decode event record list")
                }
            }
        }).expect("can not start event_handler")
}

fn mint(
    sub_api: Arc<Api>,
    signer_mnemonic_phrase: String,
    message_id: Vec<u8>,
    from: Vec<u8>,
    to: AccountId,
    amount: u128,
) {
    let signer = sr25519::Pair::from_phrase(&signer_mnemonic_phrase, None)
        .expect("invalid menemonic phrase");
    let xthex = extrinsics::build_mint(&sub_api, signer, message_id, from, to, amount);

    //send and watch extrinsic until finalized
    let _tx_hash = sub_api.send_extrinsic(xthex);
}

fn sign(
    sub_api: &Api,
    signer_mnemonic_phrase: String,
    message_id: Vec<u8>,
    from: AccountId,
    to: Vec<u8>,
    amount: u128,
) {
    let signer = sr25519::Pair::from_phrase(&signer_mnemonic_phrase, None)
        .expect("invalid menemonic phrase");
    let xthex = extrinsics::build_sign(&sub_api, signer, message_id, from, to, amount);

    //send and watch extrinsic until finalized
    let _tx_hash = sub_api.send_extrinsic(xthex);
}
