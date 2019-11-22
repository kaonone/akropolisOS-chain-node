import { Observable, from, fromEventPattern, ReplaySubject, defer } from 'rxjs';
import { switchMap, retry } from 'rxjs/operators';
import BN from 'bn.js';
import { ApiRx } from '@polkadot/api';
import { web3Enable, web3AccountsSubscribe, web3FromAddress } from '@polkadot/extension-dapp';
import { InjectedAccountWithMeta } from '@polkadot/extension-inject/types';
import { autobind } from 'core-decorators';

import { delay } from 'utils/helpers';
import { Direction, Status } from 'generated/bridge-graphql';

import { callPolkaApi } from './callPolkaApi';
import { TransfersApi } from './TransfersApi';

export class SubstrateApi {
  constructor(private apiRx: Observable<ApiRx>, private transfersApi: TransfersApi) {}

  @autobind
  public async sendToEthereum(fromAddress: string, to: string, amount: string): Promise<void> {
    const substrateApi = await this.apiRx.toPromise();
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
            this.transfersApi.pushToSubmittedTransfers$({
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

  @autobind
  // eslint-disable-next-line class-methods-use-this
  public getAccounts$(): Observable<InjectedAccountWithMeta[]> {
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

  @autobind
  public getTokenBalance$(address: string): Observable<BN> {
    return callPolkaApi(this.apiRx, 'query.token.balance', address);
  }
}
