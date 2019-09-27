import BN from 'bn.js';
import { DEFAULT_DECIMALS } from '~env';

export function formatDai(value: BN): string {
  const divisor = new BN(10).pow(new BN(DEFAULT_DECIMALS));

  const beforeDecimal = value.div(divisor).toString();
  const afterDecimal = value.mod(divisor).toString().replace(/^(.+?)0+$/, '$1');

  return `${beforeDecimal}.${afterDecimal}`;
}
