import { LimitKind } from 'generated/bridge-graphql';

export enum ConnectionStatus {
  connecting = 'CONNECTING',
  ready = 'READY',
  error = 'ERROR',
}

export interface IConnectionInfo {
  status: ConnectionStatus;
  errors: number;
}

export type ICreateProposalOptions = Record<LimitKind, string> & { fromAddress: string };
