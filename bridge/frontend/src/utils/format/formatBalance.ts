import BN from 'bn.js';

import { bnToBn } from 'utils/bn/bnToBn';

import { SI, calcSi, getSiMidIndex } from './si';
import { formatDecimal } from './formatDecimal';

interface IFormatBalanceOptions {
  amountInBaseUnits: string | BN;
  baseDecimals: number;
  tokenSymbol: string;
}

export function formatBalance({
  amountInBaseUnits,
  baseDecimals,
  tokenSymbol,
}: IFormatBalanceOptions): string {
  let balanceString = bnToBn(amountInBaseUnits).toString();

  if (balanceString.length === 0 || balanceString === '0') {
    return '0';
  }

  const isNegative = balanceString[0].startsWith('-');

  if (isNegative) {
    balanceString = balanceString.substr(1);
  }

  const si = calcSi(balanceString, baseDecimals);
  const mid = balanceString.length - (baseDecimals + si.power);
  const prefix = balanceString.substr(0, mid);
  const padding = mid < 0 ? 0 - mid : 0;

  const postfix = `${`${'0'.repeat(padding)}${balanceString}`.substr(mid < 0 ? 0 : mid)}000`.substr(
    0,
    3,
  );

  const units = si.value === '-' ? ` ${tokenSymbol}` : `${si.value} ${tokenSymbol}`;

  return `${isNegative ? '-' : ''}${formatDecimal(prefix || '0')}.${postfix}${units}`;
}

formatBalance.getOptions = (baseDecimals: number, baseUnitName?: string) => {
  const mid = getSiMidIndex();

  return SI.map((siItem, index) =>
    baseUnitName && index === mid ? { ...siItem, text: baseUnitName } : siItem,
  ).filter(({ power }): boolean => (power < 0 ? baseDecimals + power >= 0 : true));
};
