import React, { useState, useCallback } from 'react';
import { Grid, Typography } from '@material-ui/core';

import { useSubscribable } from '~util/hooks';
import { useApi } from '~components/context';
import { Address } from '~components/Address';
import SendingForm, { SendingFormProps } from './SendingForm';

function SubstrateToEthereum() {
  const api = useApi();
  const [, { error: accountError }] = useSubscribable(() => api.getSubstrateAccounts$(), []);
  const [selectedFromAddress, selectFromAddress] = useState<string | null>(null);

  const handleFormChange: NonNullable<SendingFormProps['onChange']> = useCallback(
    (values, errors) => {
      if (selectedFromAddress !== values.from) {
        !values.from && selectFromAddress(null);
        values.from && !errors.from && selectFromAddress(values.from);
      }
    },
    []
  );

  return (
    <Grid container spacing={2}>
      {!!accountError && (
        <Grid item xs={12}>
          <Typography color="error">{accountError}</Typography>
        </Grid>
      )}
      {selectedFromAddress && (
        <Grid item xs={12}>
          <Address type="substrate" address={selectedFromAddress} />
        </Grid>
      )}
      <Grid item xs={12}>
        <SendingForm onChange={handleFormChange} />
      </Grid>
    </Grid>
  );
}

export default SubstrateToEthereum;
