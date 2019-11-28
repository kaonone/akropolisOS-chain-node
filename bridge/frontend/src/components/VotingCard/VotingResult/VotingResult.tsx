import * as React from 'react';
import Typography from '@material-ui/core/Typography';
import Grid from '@material-ui/core/Grid';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { Checked, ContainedCross } from 'components/icons';
import { ProposalStatus } from 'generated/bridge-graphql';

import { useStyles } from '../VotingCard.style';

const tKeys = tKeysAll.components.votingCard;

interface IProps {
  votingStatus: ProposalStatus;
}

function VotingResult(props: IProps) {
  const { votingStatus } = props;
  const classes = useStyles();
  const { t } = useTranslate();

  return (
    <>
      <Grid container spacing={3} justify="center" direction="column">
        {(votingStatus === 'APPROVED' || votingStatus === 'DECLINED') && (
          <Grid item>
            <Grid container wrap="nowrap" alignItems="center" justify="center">
              {votingStatus === 'APPROVED' && <Checked className={classes.votingForIcon} />}
              {votingStatus === 'DECLINED' && (
                <ContainedCross className={classes.votingAgainstIcon} />
              )}
              <Typography variant="h6">
                {votingStatus === 'APPROVED'
                  ? t(tKeys.approved.getKey())
                  : t(tKeys.declined.getKey())}
              </Typography>
            </Grid>
          </Grid>
        )}
      </Grid>
    </>
  );
}

export { VotingResult };
