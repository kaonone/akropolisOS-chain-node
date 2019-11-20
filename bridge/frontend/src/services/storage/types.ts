import { Message } from 'generated/bridge-graphql';

export interface PayloadByKey {
  transactions: Message[];
  version: string;
}

export type StorageKey = keyof PayloadByKey;
