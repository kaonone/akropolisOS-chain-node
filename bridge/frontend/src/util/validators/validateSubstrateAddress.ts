import { decodeAddress } from '@polkadot/util-crypto';

export function validateSubstrateAddress(value: string): string | undefined {
  try {
    decodeAddress(value);
    return undefined;
  } catch (error) {
    console.error(error);
    return 'Enter a valid Substrate address';
  }
}
