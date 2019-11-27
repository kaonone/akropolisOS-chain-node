import * as React from 'react';
import Grid from '@material-ui/core/Grid';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';

import { VotingProgressBar } from './VotingProgressBar/VotingProgressBar';

const tKeys = tKeysAll.components.votingCard;

interface IProps {
  approveAmount: number;
  declineAmount: number;
}

function VotingProgress(props: IProps) {
  const { approveAmount, declineAmount } = props;
  const { t } = useTranslate();

  return (
    <Grid container spacing={3}>
      <Grid item xs={12}>
        <VotingProgressBar title={t(tKeys.yes.getKey())} value={approveAmount} type="for" />
      </Grid>
      <Grid item xs={12}>
        <VotingProgressBar title={t(tKeys.no.getKey())} value={declineAmount} type="against" />
      </Grid>
    </Grid>
  );
}

export { VotingProgress };
