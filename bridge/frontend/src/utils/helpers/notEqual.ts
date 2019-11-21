import * as R from 'ramda';

export const notEquals = R.pipe(R.equals, R.not);
