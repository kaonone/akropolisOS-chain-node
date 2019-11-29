import React from 'react';
import Grid from '@material-ui/core/Grid';
import Typography from '@material-ui/core/Typography';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { VotingCard } from 'components/VotingCard/VotingCard';
import { Loading, Hint } from 'components';
import {
  useLimitProposalsQuery,
  useLastValidatorsListMessageQuery,
} from 'generated/bridge-graphql';

const tKeys = tKeysAll.features.settings.limitsProposalsList;

function LimitsProposalsList() {
  const { t } = useTranslate();

  const limitsProposalsResult = useLimitProposalsQuery();
  const limitProposals = limitsProposalsResult.data?.limitProposals;

  const lastValidatorsListMessage = useLastValidatorsListMessageQuery();
  const neededVotes: number = Number(
    lastValidatorsListMessage.data?.validatorsListMessages[0]?.newHowManyValidatorsDecide || 0,
  );

  return (
    <Grid container spacing={3}>
      <Loading gqlResults={[limitsProposalsResult, lastValidatorsListMessage]}>
        {!limitProposals?.length ? (
          <Grid item xs={12}>
            <Hint>
              <Typography>{t(tKeys.notFound.getKey())}</Typography>
            </Hint>
          </Grid>
        ) : (
          limitProposals.map((limitProposal, index) => (
            <Grid key={index} item xs={6}>
              <VotingCard limitProposal={limitProposal} neededVotes={neededVotes} />
            </Grid>
          ))
        )}
      </Loading>
    </Grid>
  );
}

export { LimitsProposalsList };
