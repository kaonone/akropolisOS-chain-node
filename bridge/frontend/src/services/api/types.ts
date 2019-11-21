export type ConnectionStatus = 'CONNECTING' | 'READY' | 'ERROR';

export interface IConnectionInfo {
  status: ConnectionStatus;
  errors: number;
}
