export interface PayloadByKey {
  transactions: string;
  version: string;
}

export type StorageKey = keyof PayloadByKey;
