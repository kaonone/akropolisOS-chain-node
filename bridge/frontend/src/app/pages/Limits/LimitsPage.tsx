import * as React from 'react';

import { LimitsList, LimitsProposalsList, CreateProposalButton } from 'features/limits';
import { useLastValidatorsListMessageQuery } from 'generated/bridge-graphql';
import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { useApi } from 'services/api';
import { useSubscribable } from 'utils/react';
import { Typography, Grid } from 'components';

const tKeys = tKeysAll.app.pages.limits;

export function LimitsPage() {
  const { t } = useTranslate();
  const api = useApi();
  const [account] = useSubscribable(() => api.getEthAccount$(), []);
  const lastValidatorsListMessage = useLastValidatorsListMessageQuery();
  const canVote = !!(
    account &&
    lastValidatorsListMessage.data?.validatorsListMessages[0].newValidators.includes(account)
  );

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
          <Grid item>{canVote && <CreateProposalButton />}</Grid>
        </Grid>
      </Grid>
      <Grid item xs={12}>
        <LimitsProposalsList canVote={canVote} />
      </Grid>
    </Grid>
  );
}
