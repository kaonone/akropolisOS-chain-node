import { Observable, BehaviorSubject } from 'rxjs';
import { WsProvider } from '@polkadot/api';

import { LocalStorage } from 'services/storage';

import { IConnectionInfo } from './types';

export class ProviderApi {
  private connectionStatus = new BehaviorSubject<IConnectionInfo>({
    status: 'CONNECTING',
    errors: 0,
  });

  constructor(private storage: LocalStorage, private wsProvider: WsProvider) {
    this.setConnectionStatus();
  }

  public getNodeUrl() {
    return this.storage.get('nodeUrl');
  }

  public setNodeUrl(url: string) {
    this.storage.set('nodeUrl', url);
  }

  public getConnectionStatus$(): Observable<IConnectionInfo> {
    return this.connectionStatus;
  }

  private setConnectionStatus() {
    this.wsProvider.on('error', () => {
      const { errors } = this.connectionStatus.getValue();

      if (errors >= 5) {
        this.connectionStatus.next({
          status: 'ERROR',
          errors,
        });
      } else {
        this.connectionStatus.next({
          status: 'CONNECTING',
          errors: errors + 1,
        });
      }
    });

    this.wsProvider.on('connected', () => {
      this.connectionStatus.next({
        status: 'READY',
        errors: 0,
      });
    });
  }
}
