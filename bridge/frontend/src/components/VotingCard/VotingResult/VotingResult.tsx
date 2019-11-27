import * as React from 'react';
import Typography from '@material-ui/core/Typography';
import Grid from '@material-ui/core/Grid';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { Checked, ContainedCross } from 'components/icons';

import { VotingStatus } from '../VotingCard';
import { useStyles } from '../VotingCard.style';

const tKeys = tKeysAll.components.votingCard;

interface IProps {
  votingStatus: VotingStatus;
  approveAmount: number;
  declineAmount: number;
}

function VotingResult(props: IProps) {
  const { approveAmount, declineAmount, votingStatus } = props;
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
        <Grid item>
          <Grid container wrap="nowrap" spacing={3} justify="center">
            <Grid item>
              <Typography component="span" variant="subtitle1">
                {t(tKeys.yes.getKey())}
              </Typography>{' '}
              <Typography component="span" variant="subtitle1" className={classes.votingFor}>
                {approveAmount}
              </Typography>
            </Grid>
            <Grid item>
              <Typography component="span" variant="subtitle1">
                {t(tKeys.no.getKey())}
              </Typography>{' '}
              <Typography component="span" variant="subtitle1" className={classes.votingAgainst}>
                {declineAmount}
              </Typography>
            </Grid>
          </Grid>
        </Grid>
      </Grid>
    </>
  );
}

export { VotingResult };
