import Web3 from 'web3';
import { Observable } from 'rxjs';
import { ApiRx, WsProvider } from '@polkadot/api';

import { LocalStorage } from 'services/storage';

import { SubstrateProviderApi } from './SubstrateProviderApi';
import { EthereumApi } from './EthereumApi';
import { SubstrateApi } from './SubstrateApi';
import { TransfersApi } from './TransfersApi';

export class Api {
  private substrateProviderApi: SubstrateProviderApi;
  private ethereumApi: EthereumApi;
  private substrateApi: SubstrateApi;
  private transfersApi: TransfersApi;

  constructor(
    private web3: Web3,
    private apiRx: Observable<ApiRx>,
    private storage: LocalStorage,
    private wsProvider: WsProvider,
  ) {
    this.transfersApi = new TransfersApi(this.storage);
    this.substrateProviderApi = new SubstrateProviderApi(this.storage, this.wsProvider);
    this.ethereumApi = new EthereumApi(this.web3, this.transfersApi);
    this.substrateApi = new SubstrateApi(this.apiRx, this.transfersApi);
  }

  get sendToEthereum() {
    return this.substrateApi.sendToEthereum;
  }

  get sendToSubstrate() {
    return this.ethereumApi.sendToSubstrate;
  }

  get getTransfers$() {
    return this.transfersApi.getTransfers$;
  }

  get getEthValidators$() {
    return this.ethereumApi.getEthValidators$;
  }

  get getNeededLimitsVotes$() {
    return this.ethereumApi.getNeededLimitsVotes$;
  }

  get getEthTokenBalance$() {
    return this.ethereumApi.getTokenBalance$;
  }

  get getSubstrateTokenBalance$() {
    return this.substrateApi.getTokenBalance$;
  }

  get getEthAccount$() {
    return this.ethereumApi.getAccount$;
  }

  get getSubstrateAccounts$() {
    return this.substrateApi.getAccounts$;
  }

  get getNodeUrl() {
    return this.substrateProviderApi.getNodeUrl;
  }

  get setNodeUrl() {
    return this.substrateProviderApi.setNodeUrl;
  }

  get getConnectionStatus$() {
    return this.substrateProviderApi.getConnectionStatus$;
  }
}
