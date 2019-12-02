export enum ConnectionStatus {
  connecting = 'CONNECTING',
  ready = 'READY',
  error = 'ERROR',
}

export interface IConnectionInfo {
  status: ConnectionStatus;
  errors: number;
}

export interface ICreateProposalOptions {
  fromAddress: string;
  minHostTransactionValue: string;
  maxHostTransactionValue: string;
  dayHostMaxLimit: string;
  dayHostMaxLimitForOneAddress: string;
  maxHostPendingTransactionLimit: string;
  minGuestTransactionValue: string;
  maxGuestTransactionValue: string;
  dayGuestMaxLimit: string;
  dayGuestMaxLimitForOneAddress: string;
  maxGuestPendingTransactionLimit: string;
}
