import { checkAddress } from '@polkadot/util-crypto';

import { tKeys, ITranslateKey } from 'services/i18n';
import { SUBSTRATE_DEFAULT_ADDRESS_PREFIX } from 'env';

export function validateSubstrateAddress(value: string): ITranslateKey | undefined {
  try {
    const [isValid, error] = checkAddress(value, SUBSTRATE_DEFAULT_ADDRESS_PREFIX);
    if (!isValid && error) {
      throw new Error(error);
    }
    return undefined;
  } catch (error) {
    return tKeys.utils.validation.isValidSubstrateAddress.getKey();
  }
}
