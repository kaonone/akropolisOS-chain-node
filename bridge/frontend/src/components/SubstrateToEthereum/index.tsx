import * as React from 'react';
import { Grid, Typography } from '@material-ui/core';

import { useSubscribable } from '~util/hooks';
import { useApi } from '~components/context';
import { Balance } from '~components/Balance';

function SubstrateToEthereum() {
  const api = useApi();
  const [accounts, { error: accountError }] = useSubscribable(() => api.getSubstrateAccounts$(), []);

  return (
    <Grid container spacing={2}>
      {!!accountError && (
        <Grid item xs={12}>
          <Typography color="error">{accountError}</Typography>
        </Grid>
      )}
      {accounts && accounts.map(account => (
        <Grid item xs={12}>
          <Balance type="substrate" address={account.address} name={account.meta.name} />
        </Grid>
      ))}
      <Grid item xs={12}>
        Coming Soon
      </Grid>
    </Grid>
  );
}

export default SubstrateToEthereum;
