import * as React from 'react';
import { Grid, Typography } from '@material-ui/core';

import { useSubscribable } from '~util/hooks';
import { useApi } from '~components/context';
import { Balance } from '~components/Balance';

import SendingForm from './SendingForm';

function EthereumToSubstrate() {
  const api = useApi();
  const [account, { error: accountError }] = useSubscribable(() => api.getEthAccount$(), []);

  return (
    <Grid container spacing={2}>
      <Grid item xs={12}>
        {!!accountError && <Typography color="error">{accountError}</Typography>}
        {account && <Balance address={account} type="ethereum" />}
      </Grid>
      <Grid item xs={12}>
        <SendingForm />
      </Grid>
    </Grid>
  );
}

export default EthereumToSubstrate;
