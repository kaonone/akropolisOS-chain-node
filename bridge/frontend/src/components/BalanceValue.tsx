import BN from 'bn.js';
import * as React from 'react';

import { formatBalance } from 'utils/format';
import { DEFAULT_DECIMALS, ETHEREUM_UNIT_NAME } from 'env';

interface IProps {
  input: string | BN;
  withSi?: boolean;
  decimals?: number;
}

function BalanceValue(props: IProps) {
  const { input } = props;

  return (
    <>
      {formatBalance({
        amountInBaseUnits: input,
        baseDecimals: DEFAULT_DECIMALS,
        tokenSymbol: ETHEREUM_UNIT_NAME,
      })}
    </>
  );
}

export { BalanceValue };
