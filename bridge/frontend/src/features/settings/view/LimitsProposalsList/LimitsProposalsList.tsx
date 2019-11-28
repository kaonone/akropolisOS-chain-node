import React from 'react';
import Grid from '@material-ui/core/Grid';
import Typography from '@material-ui/core/Typography';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { useApi } from 'services/api';
import { Loading, Hint } from 'components';
import { VotingCard } from 'components/VotingCard/VotingCard';
import { useLimitProposalsQuery } from 'generated/bridge-graphql';
import { useSubscribable } from 'utils/react';

const tKeys = tKeysAll.features.settings.limitsProposalsList;

function LimitsProposalsList() {
  const { t } = useTranslate();

  const limitsProposalsResult = useLimitProposalsQuery();
  const limitProposals = limitsProposalsResult.data?.limitProposals;

  const api = useApi();
  const [neddedVotes, neddedVotesMeta] = useSubscribable(() => api.getNeededLimitsVotes$(), [], 0);

  return (
    <>
      <Typography variant="h4" noWrap gutterBottom>
        {t(tKeys.title.getKey())}
      </Typography>
      <Grid container spacing={3}>
        <Loading gqlResults={limitsProposalsResult} meta={neddedVotesMeta}>
          {!limitProposals || !limitProposals.length ? (
            <Grid item xs={12}>
              <Hint>
                <Typography>{t(tKeys.notFound.getKey())}</Typography>
              </Hint>
            </Grid>
          ) : (
            limitProposals.map((limitProposal, index) => (
              <Grid key={index} item xs={12}>
                <VotingCard limitProposal={limitProposal} neddedVotes={neddedVotes} />
              </Grid>
            ))
          )}
        </Loading>
      </Grid>
    </>
  );
}

export { LimitsProposalsList };
