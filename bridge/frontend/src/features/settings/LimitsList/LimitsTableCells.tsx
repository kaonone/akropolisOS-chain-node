import React from 'react';

export function KindCell({ kind }: { kind: string }) {
  return <>{kind}</>;
}

export function ValueCell({ value }: { value: string }) {
  return <>{value}</>;
}

export function EthBlockNumberCell({ blockNumber }: { blockNumber: string }) {
  return <>{blockNumber}</>;
}
