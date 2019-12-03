import * as React from 'react';

import { LimitsList, LimitsProposalsList, CreateProposalButton } from 'features/limits';
import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { Typography, Grid } from 'components';

const tKeys = tKeysAll.app.pages.limits;

export function LimitsPage() {
  const { t } = useTranslate();

  return (
    <Grid container spacing={3}>
      <Grid item xs={12}>
        <Typography variant="h4" noWrap gutterBottom>
          {t(tKeys.title.getKey())}
        </Typography>
      </Grid>
      <Grid item xs={12}>
        <LimitsList />
      </Grid>
      <Grid item xs={12}>
        <Grid container spacing={2}>
          <Grid item>
            <Typography variant="h4" noWrap gutterBottom>
              {t(tKeys.proposalsTitle.getKey())}
            </Typography>
          </Grid>
          <Grid item>
            <CreateProposalButton canVote />
          </Grid>
        </Grid>
      </Grid>
      <Grid item xs={12}>
        <LimitsProposalsList />
      </Grid>
    </Grid>
  );
}
