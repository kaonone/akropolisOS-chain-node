import React from 'react';
import Grid from '@material-ui/core/Grid';
import Typography from '@material-ui/core/Typography';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { VotingCard } from 'components/VotingCard/VotingCard';

const tKeys = tKeysAll.features.settings.limitsProposalsList;

function LimitsProposalsList() {
  const { t } = useTranslate();

  const mockIds = ['123', '456', '789', '1231', '124124'];

  return (
    <>
      <Typography variant="h4" noWrap gutterBottom>
        {t(tKeys.title.getKey())}
      </Typography>
      <Grid container spacing={3}>
        {mockIds.map(id => (
          <Grid key={id} item xs={12}>
            <VotingCard id={id} needVoted={200} showLimitsList />
          </Grid>
        ))}
      </Grid>
    </>
  );
}

export { LimitsProposalsList };
