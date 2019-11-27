import React from 'react';
import Grid from '@material-ui/core/Grid';
import Typography from '@material-ui/core/Typography';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { VotingCard } from 'components/VotingCard/VotingCard';

const tKeys = tKeysAll.features.settings.limitsProposalsList;

function LimitsProposalsList() {
  const { t } = useTranslate();

  return (
    <Grid container>
      <Grid item xs={8}>
        <Typography variant="h4" noWrap gutterBottom>
          {t(tKeys.title.getKey())}
        </Typography>
      </Grid>
      <Grid item xs={8}>
        <VotingCard
          ethBlockNumber="123456"
          fromAddress="123456"
          votingStatus="APPROVED"
          approveAmount={60}
          declineAmount={70}
          voted={150}
          needVoted={200}
          timeLeft="15 min"
          showLimitsList
        />
      </Grid>
    </Grid>
  );
}

export { LimitsProposalsList };
