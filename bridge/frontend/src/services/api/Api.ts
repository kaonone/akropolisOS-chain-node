import Web3 from 'web3';
import { Observable } from 'rxjs';
import BN from 'bn.js';
import { ApiRx, WsProvider } from '@polkadot/api';
import { InjectedAccountWithMeta } from '@polkadot/extension-inject/types';

import { Message } from 'generated/bridge-graphql';
import { LocalStorage } from 'services/storage';

import { IConnectionInfo } from './types';
import { ProviderApi } from './ProviderApi';
import { EthereumApi } from './EthereumApi';
import { SubstrateApi } from './SubstrateApi';
import { TransactionsApi } from './TransactionsApi';

export class Api {
  private providerApi: ProviderApi;
  private ethereumApi: EthereumApi;
  private substrateApi: SubstrateApi;
  private transactionsApi: TransactionsApi;

  constructor(
    private web3: Web3,
    private apiRx: Observable<ApiRx>,
    private storage: LocalStorage,
    private wsProvider: WsProvider,
  ) {
    this.transactionsApi = new TransactionsApi(this.storage);
    this.providerApi = new ProviderApi(this.storage, this.wsProvider);
    this.ethereumApi = new EthereumApi(this.web3, this.transactionsApi);
    this.substrateApi = new SubstrateApi(this.apiRx, this.transactionsApi);
  }

  public async sendToEthereum(fromAddress: string, to: string, amount: string): Promise<void> {
    return this.substrateApi.sendToEthereum(fromAddress, to, amount);
  }

  public async sendToSubstrate(fromAddress: string, to: string, amount: string): Promise<void> {
    return this.ethereumApi.sendToSubstrate(fromAddress, to, amount);
  }

  public getTransactions$(): Observable<Message[]> {
    return this.transactionsApi.getTransactions$();
  }

  public getEthValidators$(): Observable<string[]> {
    return this.ethereumApi.getEthValidators$();
  }

  public getEthBalance$(address: string): Observable<BN> {
    return this.ethereumApi.getEthBalance$(address);
  }

  public getSubstrateBalance$(address: string): Observable<BN> {
    return this.substrateApi.getSubstrateBalance$(address);
  }

  public getEthAccount$(): Observable<string | null> {
    return this.ethereumApi.getEthAccount$();
  }

  public getSubstrateAccounts$(): Observable<InjectedAccountWithMeta[]> {
    return this.substrateApi.getSubstrateAccounts$();
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
