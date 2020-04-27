use akropolisos_runtime::types::Token;
use akropolisos_runtime::{
    AccountId, AuraConfig, BalancesConfig, BridgeConfig, GenesisConfig, GrandpaConfig, Signature,
    SudoConfig, SystemConfig, TokenConfig, WASM_BINARY,
};
use hex_literal::hex;
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate an authority key for Aura
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Development,
        || {
            testnet_genesis(
                vec![authority_keys_from_seed("Alice")],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                true,
            )
        },
        vec![],
        None,
        None,
        None,
        None,
    )
}

pub fn local_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Local Testnet",
        "local_testnet",
        ChainType::Local,
        || {
            testnet_genesis(
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                true,
            )
        },
        vec![],
        None,
        None,
        None,
        None,
    )
}

pub fn syracuse_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Akropolis OS Syracuse Testnet",
        "akropolisos_syracuse_testnet",
        ChainType::Custom("Syracuse".into()),
        || {
            testnet_genesis(
                vec![
                    authority_keys_from_seed("Akropolis1"),
                    authority_keys_from_seed("Akropolis2"),
                ],
                hex!("0d96d3dbdb55964e521a2f1dc1428ae55336063fd8f0e07bebbcb1becf79a67b").into(),
                // 5CtXvt2othnZpkneuTg6xENMwXbmwV3da1YeNAeYx5wMaCvz
                vec![
                    hex!("0d96d3dbdb55964e521a2f1dc1428ae55336063fd8f0e07bebbcb1becf79a67b").into(),
                    // 5CtXvt2othnZpkneuTg6xENMwXbmwV3da1YeNAeYx5wMaCvz
                    hex!("80133ea92f48aa928119aaaf524bc75e436a5c9eb24878a9e28ac7b0b37aa81a").into(),
                    // 5CqXmy44eTwGQCX8GaLrUfTAyEswGSd4PgSKMgUdLfDLBhZZ
                    hex!("3c7f612cdda6d0a3aad9da0fb6cb624721b04067f00bd0034062e6e2db2cd23e").into(),
                    // 5DnUF5fQ6KNYPWRAcHYpMu32pUtdLv6ksRcSLeuofrxmPsTU
                ],
                true,
            )
        },
        vec![],
        None,
        None,
        None,
        None,
    )
}

fn testnet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    println!("Initial AuthoritiesA:{:?}\nEndowed Accounts: {:?}",initial_authorities, endowed_accounts);
    let tokens = vec![
        Token {
            id: 0,
            decimals: 18,
            symbol: Vec::from("DAI"),
        },
        Token {
            id: 1,
            decimals: 18,
            symbol: Vec::from("cDAI"),
        },
        Token {
            id: 2,
            decimals: 18,
            symbol: Vec::from("USDT"),
        },
        Token {
            id: 3,
            decimals: 18,
            symbol: Vec::from("USDC"),
        },
    ];

    GenesisConfig {
        system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        }),
        aura: Some(AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        }),
        grandpa: Some(GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        }),
        sudo: Some(SudoConfig { key: root_key }),
        bridge: Some(BridgeConfig {
            validator_accounts: endowed_accounts,
            validators_count: 3u32,
            current_limits: vec![
                100 * 10u128.pow(18),
                200 * 10u128.pow(18),
                50 * 10u128.pow(18),
                400 * 10u128.pow(18),
                10 * 10u128.pow(18),
            ],
        }),
        dao: None,
        token: Some(TokenConfig { tokens }),
    }
}
