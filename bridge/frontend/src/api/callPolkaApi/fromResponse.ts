import { FromResponseConverters } from './types';

export const fromResponseConverters: FromResponseConverters = {
  'query.token.balance_of': response => response.toBn(),
};
