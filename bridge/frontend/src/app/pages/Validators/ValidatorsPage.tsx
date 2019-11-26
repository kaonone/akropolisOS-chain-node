import * as React from 'react';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { Typography, Hint, EthereumValidators, Grid } from 'components';

const tKeys = tKeysAll.app.pages.validators;

export function ValidatorsPage() {
  const { t } = useTranslate();

  return (
    <Grid container spacing={3}>
      <Grid item xs={12}>
        <Typography variant="h4" noWrap gutterBottom>
          {t(tKeys.title.getKey())}
        </Typography>
      </Grid>
      <Grid item xs={12}>
        <EthereumValidators />
      </Grid>
      <Grid item xs={12}>
        <Hint>Coming soon</Hint>
      </Grid>
    </Grid>
  );
}
