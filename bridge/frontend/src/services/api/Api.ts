import * as R from 'ramda';
import Web3 from 'web3';
import Contract from 'web3/eth/contract';
import {
  Observable,
  interval,
  from,
  fromEventPattern,
  ReplaySubject,
  defer,
  BehaviorSubject,
} from 'rxjs';
import { switchMap, skipWhile, retry } from 'rxjs/operators';
import BN from 'bn.js';
import { ApiRx, WsProvider } from '@polkadot/api';
import { web3Enable, web3AccountsSubscribe, web3FromAddress } from '@polkadot/extension-dapp';
import { InjectedAccountWithMeta } from '@polkadot/extension-inject/types';
import { decodeAddress } from '@polkadot/util-crypto';
import { u8aToHex } from '@polkadot/util';

import { ETH_NETWORK_CONFIG } from 'env';
import bridgeAbi from 'abis/bridge.json';
import erc20Abi from 'abis/erc20.json';
import { getContractData$ } from 'utils/ethereum';
import { delay } from 'utils/helpers';
import { LocalStorage } from 'services/storage';
import { Direction, Message, Status } from 'generated/bridge-graphql';

import { callPolkaApi } from './callPolkaApi';
import { IConnectionInfo } from './types';
import { ProviderApi } from './ProviderApi';

export class Api {
  private daiContract: Contract;
  private bridgeContract: Contract;
  private submittedTransactions = new BehaviorSubject<Message[]>([]);
  private connectionStatus = new BehaviorSubject<IConnectionInfo>({
    status: 'CONNECTING',
    errors: 0,
  });

  private providerApi: ProviderApi;

  constructor(
    private web3: Web3,
    private substrateApi: Observable<ApiRx>,
    private storage: LocalStorage,
    private wsProvider: WsProvider,
  ) {
    this.daiContract = new this.web3.eth.Contract(erc20Abi, ETH_NETWORK_CONFIG.contracts.dai);
    this.bridgeContract = new this.web3.eth.Contract(
      bridgeAbi,
      ETH_NETWORK_CONFIG.contracts.bridge,
    );
    this.initTransactions();

    this.providerApi = new ProviderApi(this.storage, this.wsProvider);
  }

  public async sendToEthereum(fromAddress: string, to: string, amount: string): Promise<void> {
    const substrateApi = await this.substrateApi.toPromise();
    const substrateWeb3 = await web3FromAddress(fromAddress);
    substrateApi.setSigner(substrateWeb3.signer);

    const transfer = substrateApi.tx.bridge.setTransfer(to, amount);

    await new Promise((resolve, reject) => {
      transfer.signAndSend(fromAddress).subscribe({
        complete: resolve,
        error: reject,
        next: ({ isCompleted, isError, events }) => {
          const failedEvent = events.find(
            event => event.event.meta.name.toString() === 'ExtrinsicFailed',
          );
          const messageHashEvent = events.find(
            event => event.event.meta.name.toString() === 'RelayMessage',
          );

          const id = messageHashEvent && messageHashEvent.event.data[0]?.toHex();

          if (id) {
            this.pushToSubmittedTransactions$({
              id,
              amount,
              direction: Direction.Sub2Eth,
              ethAddress: to,
              subAddress: fromAddress,
              status: Status.Pending,
            });
          }

          (isError || failedEvent) &&
            reject(new Error('tx.bridge.setTransfer extrinsic is failed'));
          isCompleted && id && resolve(id);
          isCompleted && !id && reject(new Error('Message ID is not found'));
        },
      });
    });
  }

  public async sendToSubstrate(fromAddress: string, to: string, amount: string): Promise<void> {
    await this.approveBridge(fromAddress, amount);
    await this.sendToBridge(fromAddress, to, amount);
  }

  private async approveBridge(fromAddress: string, amount: string): Promise<void> {
    const allowance: string = await this.daiContract.methods
      .allowance(fromAddress, ETH_NETWORK_CONFIG.contracts.bridge)
      .call();

    if (new BN(amount).lte(new BN(allowance))) {
      return;
    }

    await this.daiContract.methods
      .approve(ETH_NETWORK_CONFIG.contracts.bridge, amount)
      .send({ from: fromAddress });
  }

  private async sendToBridge(fromAddress: string, to: string, amount: string): Promise<void> {
    const formatedToAddress = u8aToHex(decodeAddress(to));
    const bytesAddress = this.web3.utils.hexToBytes(formatedToAddress);

    const result = await this.bridgeContract.methods
      .setTransfer(amount, bytesAddress)
      .send({ from: fromAddress });

    const id = result?.events?.RelayMessage?.returnValues?.messageID;

    id &&
      this.pushToSubmittedTransactions$({
        id,
        amount,
        direction: Direction.Eth2Sub,
        ethAddress: fromAddress,
        subAddress: to,
        status: Status.Pending,
      });
  }

  private initTransactions() {
    const prevMessages = this.storage.get('transactions', []);
    this.submittedTransactions.next(prevMessages);
  }

  private pushToSubmittedTransactions$(transactionInfo: Message) {
    const prevTransactions = this.storage.get('transactions', []);

    const transactions = R.uniq([...prevTransactions, transactionInfo]);

    this.storage.set('transactions', transactions);
    this.submittedTransactions.next(transactions);
  }

  public getTransactions$() {
    return this.submittedTransactions;
  }

  // eslint-disable-next-line class-methods-use-this
  public getEthValidators$(): Observable<string[]> {
    return from([
      [
        '6a8357ae0173737209af59152ee30a786dbade70',
        '93880d6508e3ffee5a4376939d3322f2f11b56d1',
        '9194ad793e72052992f9a1b3b8eaef5463300f87',
      ],
    ]);

    /* return getContractData$<string[], string[]>(this._bridgeContract, 'validators', {
      eventsForReload: [['ValidatorShipTransferred']],
    }); */
  }

  public getEthBalance$(address: string): Observable<BN> {
    const formattedAddress = address.toLowerCase();

    return getContractData$<string, BN>(this.daiContract, 'balanceOf', {
      args: [formattedAddress],
      eventsForReload: [
        ['Transfer', { filter: { _from: formattedAddress } }],
        ['Transfer', { filter: { _to: formattedAddress } }],
      ],
      convert: value => new BN(value),
    });
  }

  public getSubstrateBalance$(address: string): Observable<BN> {
    return callPolkaApi(this.substrateApi, 'query.token.balance', address);
  }

  public getEthAccount$(): Observable<string | null> {
    return from(getEthAccount(this.web3)).pipe(
      skipWhile(account => !account),
      switchMap(() => interval(1000).pipe(switchMap(() => getEthAccount(this.web3)))),
    );
  }

  // eslint-disable-next-line class-methods-use-this
  public getSubstrateAccounts$(): Observable<InjectedAccountWithMeta[]> {
    const accounts$ = new ReplaySubject<InjectedAccountWithMeta[]>();

    defer(() =>
      from(
        (async () => {
          const injected = await web3Enable('Substrate <-> Ethereum Bridge');
          if (!injected.length) {
            await delay(1000);
          }
          return injected;
        })(),
      ),
    )
      .pipe(
        switchMap(injectedExtensions =>
          injectedExtensions.length
            ? fromEventPattern<InjectedAccountWithMeta[]>(
                emitter => web3AccountsSubscribe(emitter),
                (_, signal: ReturnType<typeof web3AccountsSubscribe>) =>
                  signal.then(unsubscribe => unsubscribe()),
              )
            : new Observable<InjectedAccountWithMeta[]>(subscriber =>
                subscriber.error(new Error('Injected extensions not found')),
              ),
        ),
        retry(3),
      )
      .subscribe(accounts$);

    return accounts$;
  }

  public getNodeUrl() {
    return this.providerApi.getNodeUrl();
  }

  public setNodeUrl(url: string) {
    this.providerApi.setNodeUrl(url);
  }

  public getConnectionStatus$(): Observable<IConnectionInfo> {
    return this.providerApi.getConnectionStatus$();
  }
}

async function getEthAccount(web3: Web3): Promise<string | null> {
  // Modern dapp browsers...
  if (window.ethereum) {
    try {
      // Request account access
      await window.ethereum.enable();
    } catch (error) {
      console.error('User denied account access');
      throw error;
    }
  }

  const accounts = await web3.eth.getAccounts();
  return accounts[0] || null;
}
