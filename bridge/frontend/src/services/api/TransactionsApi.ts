import * as R from 'ramda';
import { BehaviorSubject, Observable } from 'rxjs';

import { LocalStorage } from 'services/storage';
import { Message } from 'generated/bridge-graphql';

export class TransactionsApi {
  private submittedTransactions = new BehaviorSubject<Message[]>([]);

  constructor(private storage: LocalStorage) {
    this.initTransactions();
  }

  public pushToSubmittedTransactions$(transactionInfo: Message) {
    const prevTransactions = this.storage.get('transactions', []);

    const transactions = R.uniq([...prevTransactions, transactionInfo]);

    this.storage.set('transactions', transactions);
    this.submittedTransactions.next(transactions);
  }

  public getTransactions$(): Observable<Message[]> {
    return this.submittedTransactions;
  }

  private initTransactions() {
    const prevMessages = this.storage.get('transactions', []);
    this.submittedTransactions.next(prevMessages);
  }
}
