import * as R from 'ramda';
import { BehaviorSubject, Observable } from 'rxjs';
import { autobind } from 'core-decorators';

import { LocalStorage } from 'services/storage';
import { Message } from 'generated/bridge-graphql';

export class TransfersApi {
  private submittedTransfers = new BehaviorSubject<Message[]>([]);

  constructor(private storage: LocalStorage) {
    this.initTransfers();
  }

  public pushToSubmittedTransfers$(transferInfo: Message) {
    const prevTransfer = this.storage.get('transfers', []);

    const transfers = R.uniq([...prevTransfer, transferInfo]);

    this.storage.set('transfers', transfers);
    this.submittedTransfers.next(transfers);
  }

  @autobind
  public getTransfers$(): Observable<Message[]> {
    return this.submittedTransfers;
  }

  private initTransfers() {
    const prevMessages = this.storage.get('transfers', []);
    this.submittedTransfers.next(prevMessages);
  }
}
