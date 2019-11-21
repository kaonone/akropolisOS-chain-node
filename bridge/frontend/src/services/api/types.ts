export enum ConnectionStatus {
  connecting = 'CONNECTING',
  ready = 'READY',
  error = 'ERROR',
}

export interface IConnectionInfo {
  status: ConnectionStatus;
  errors: number;
}
