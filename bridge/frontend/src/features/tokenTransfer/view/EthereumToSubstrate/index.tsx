import * as React from 'react';

import { Grid, Typography, Address, EthereumValidators } from 'components';
import { useSubscribable } from 'utils/hooks';
import { useApi } from 'services/api';

import { SendingForm } from './SendingForm';

function EthereumToSubstrate() {
  const api = useApi();
  const [account, { error: accountError }] = useSubscribable(() => api.getEthAccount$(), []);

  return (
    <Grid container spacing={2}>
      <Grid item xs={12}>
        {!!accountError && <Typography color="error">{accountError}</Typography>}
        {account && <Address address={account} type="ethereum" />}
      </Grid>
      <Grid item xs={12}>
        <SendingForm />
      </Grid>
      <Grid item xs={12}>
        <EthereumValidators />
      </Grid>
    </Grid>
  );
}

export { EthereumToSubstrate };
