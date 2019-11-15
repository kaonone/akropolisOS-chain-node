import Web3 from 'web3';

import { tKeys, ITranslateKey } from 'services/i18n';

export function validateEthereumAddress(value: string): ITranslateKey | undefined {
  return value && Web3.utils.isAddress(value.toLowerCase())
    ? undefined
    : tKeys.utils.validation.isValidEthereumAddress.getKey();
}
