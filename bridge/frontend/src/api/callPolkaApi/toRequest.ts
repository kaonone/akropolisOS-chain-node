import { ToRequestConverters } from './types';
import { GenericAccountId } from '@polkadot/types';

export const toRequestConverters: ToRequestConverters = {
  'query.token.balance_of': address => new GenericAccountId(address),
};
