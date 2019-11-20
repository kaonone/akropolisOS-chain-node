export interface PayloadByKey {
  transactions: string;
  version: string;
  nodeUrl: string;
}

export type StorageKey = keyof PayloadByKey;
