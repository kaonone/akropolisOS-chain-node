import React, { useState, useCallback } from 'react';

import { Grid, Address } from 'components';

import { SendingFormProps, SendingForm } from './SendingForm';

function SubstrateToEthereum() {
  const [selectedFromAddress, selectFromAddress] = useState<string | null>(null);

  const handleFormChange: NonNullable<SendingFormProps['onChange']> = useCallback(
    (values, errors) => {
      if (selectedFromAddress !== values.from) {
        !values.from && selectFromAddress(null);
        values.from && !errors.from && selectFromAddress(values.from);
      }
    },
    [],
  );

  return (
    <Grid container spacing={2}>
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

export { SubstrateToEthereum };
