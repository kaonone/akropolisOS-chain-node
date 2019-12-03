import React from 'react';
import Grid from '@material-ui/core/Grid';
import Typography from '@material-ui/core/Typography';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { useApi } from 'services/api';
import { VotingCard } from 'components/VotingCard/VotingCard';
import { Loading, Hint } from 'components';
import { useLimitProposalsQuery } from 'generated/bridge-graphql';
import { useSubscribable } from 'utils/react';

import { LimitsList } from '../LimitsList/LimitsList';
import { VoteButton } from '../VoteButton/VoteButton';

const tKeys = tKeysAll.features.limits.limitsProposalsList;

function LimitsProposalsList() {
  const api = useApi();
  const [account, accountMeta] = useSubscribable(() => api.getEthAccount$(), []);

  const { t } = useTranslate();

  const limitsProposalsResult = useLimitProposalsQuery();
  const limitProposals = limitsProposalsResult.data?.limitProposals;

  return (
    <Grid container spacing={3}>
      <Loading gqlResults={limitsProposalsResult} meta={accountMeta}>
        {!limitProposals?.length ? (
          <Grid item xs={12}>
            <Hint>
              <Typography>{t(tKeys.notFound.getKey())}</Typography>
            </Hint>
          </Grid>
        ) : (
          account &&
          limitProposals.map(({ id, ethBlockNumber, ethAddress, status }, index) => (
            <Grid key={index} item xs={6}>
              <VotingCard
                ethBlockNumber={ethBlockNumber}
                ethAddress={ethAddress}
                status={status}
                expansionPanelTitle={t(tKeys.showLimits.getKey())}
                expansionPanelDetails={<LimitsList variant="compact" />}
              >
                <VotingCard.Voting>
                  <VoteButton proposalId={id} fromAddress={account} />
                </VotingCard.Voting>
              </VotingCard>
            </Grid>
          ))
        )}
      </Loading>
    </Grid>
  );
}

export { LimitsProposalsList };
