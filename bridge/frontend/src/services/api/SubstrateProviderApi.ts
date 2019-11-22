import { Observable, BehaviorSubject } from 'rxjs';
import { WsProvider } from '@polkadot/api';
import { autobind } from 'core-decorators';

import { LocalStorage } from 'services/storage';

import { IConnectionInfo, ConnectionStatus } from './types';

export class SubstrateProviderApi {
  private connectionStatus = new BehaviorSubject<IConnectionInfo>({
    status: ConnectionStatus.connecting,
    errors: 0,
  });

  constructor(private storage: LocalStorage, private wsProvider: WsProvider) {
    this.listenConnectionStatus();
  }

  @autobind
  public getNodeUrl() {
    return this.storage.get('nodeUrl');
  }

  @autobind
  public setNodeUrl(url: string) {
    this.storage.set('nodeUrl', url);
  }

  @autobind
  public getConnectionStatus$(): Observable<IConnectionInfo> {
    return this.connectionStatus;
  }

  private listenConnectionStatus() {
    this.wsProvider.on('error', () => {
      const { errors } = this.connectionStatus.getValue();

      if (errors >= 5) {
        this.connectionStatus.next({
          status: ConnectionStatus.error,
          errors,
        });
      } else {
        this.connectionStatus.next({
          status: ConnectionStatus.connecting,
          errors: errors + 1,
        });
      }
    });

    this.wsProvider.on('connected', () => {
      this.connectionStatus.next({
        status: ConnectionStatus.ready,
        errors: 0,
      });
    });
  }
}
