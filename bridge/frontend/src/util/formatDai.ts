import BN from 'bn.js';
import { DEFAULT_DECIMALS } from '~env';

export function formatDai(value: BN): string {
  const divisor = new BN(10).pow(new BN(DEFAULT_DECIMALS));

  const beforeDecimal = value.div(divisor).toString();
  const afterDecimal = value.mod(divisor).toString();
  
  const zeroSymbol = '0';
  const additionalZeroSymbolsCount =Math.max(DEFAULT_DECIMALS - afterDecimal.length, 0);

  const afterDecimalCompact = afterDecimal.replace(/^(.+?)0+$/, '$1');

  return `${beforeDecimal}.${zeroSymbol.repeat(additionalZeroSymbolsCount)}${afterDecimalCompact}`;
}
