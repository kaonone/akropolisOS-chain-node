import * as React from 'react';
import Grid from '@material-ui/core/Grid';
import Typography from '@material-ui/core/Typography';
import ExpansionPanel from '@material-ui/core/ExpansionPanel';
import ExpansionPanelSummary from '@material-ui/core/ExpansionPanelSummary';
import ExpansionPanelDetails from '@material-ui/core/ExpansionPanelDetails';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { ShortAddress } from 'components/ShortAddress/ShortAddress';
import { ContainedCircleArrow, OutlinedCircleArrow } from 'components/icons';
import { LimitsList } from 'features/settings';

import { VotingProgress } from './VotingProgress/VotingProgress';
import { VotingResult } from './VotingResult/VotingResult';
import { useStyles } from './VotingCard.style';
import { Column } from './Column/Column';

const tKeys = tKeysAll.components.votingCard;

export type VotingStatus = 'PENDING' | 'APPROVED' | 'DECLINED';

interface IOwnProps {
  ethBlockNumber: string;
  fromAddress: string;
  votingStatus: VotingStatus;
  approveAmount: number;
  declineAmount: number;
  voted: number;
  needVoted: number;
  timeLeft: string;
  showLimitsList: boolean;
}

function VotingCard(props: IOwnProps) {
  const {
    ethBlockNumber,
    fromAddress,
    votingStatus,
    approveAmount,
    declineAmount,
    voted,
    needVoted,
    showLimitsList,
    timeLeft,
  } = props;

  const classes = useStyles();
  const { t } = useTranslate();
  const [expanded, setExpanded] = React.useState(false);
  const isOver = votingStatus !== 'PENDING';
  const timeLeftTitle = isOver ? t(tKeys.timeEnded.getKey()) : t(tKeys.timeLeft.getKey());

  const handleExpansionPanelChange = (_event: React.ChangeEvent<{}>, isExpanded: boolean) => {
    setExpanded(isExpanded);
  };

  return (
    <Grid className={classes.root} container wrap="nowrap">
      <Grid item xs={9} className={classes.mainInformation}>
        <Grid container spacing={3}>
          <Column xs={3} title="Block number" value={ethBlockNumber} isHighlighted />
          <Column
            xs={3}
            title="from"
            value={<ShortAddress className={classes.address} address={fromAddress} />}
            isHighlighted
          />
          <Column
            xs={3}
            title={t(tKeys.voted.getKey())}
            value={voted}
            subValue={`${needVoted} ${t(tKeys.needed.getKey())}`}
            isHighlighted
          />
          <Column xs={3} title={timeLeftTitle} value={timeLeft} isHighlighted />
          {showLimitsList && (
            <Grid item xs={12} zeroMinWidth container wrap="nowrap">
              <ExpansionPanel
                onChange={handleExpansionPanelChange}
                className={classes.expansionPanel}
              >
                <ExpansionPanelSummary
                  className={classes.expansionPanelSummary}
                  aria-controls="panel1a-content"
                  id="panel1a-header"
                >
                  {expanded && <ContainedCircleArrow className={classes.toggleExpandIcon} />}
                  {!expanded && <OutlinedCircleArrow className={classes.toggleExpandIcon} />}
                  <Typography className={classes.showLimits}>
                    {t(tKeys.showLimits.getKey())}
                  </Typography>
                </ExpansionPanelSummary>
                <ExpansionPanelDetails>
                  <LimitsList />
                </ExpansionPanelDetails>
              </ExpansionPanel>
            </Grid>
          )}
        </Grid>
      </Grid>
      {!isOver && (
        <Grid item xs={3} className={classes.voting}>
          <VotingProgress approveAmount={approveAmount} declineAmount={declineAmount} />
        </Grid>
      )}
      {isOver && (
        <Grid item xs={3} className={classes.votingResult}>
          <VotingResult
            votingStatus={votingStatus}
            approveAmount={approveAmount}
            declineAmount={declineAmount}
          />
        </Grid>
      )}
    </Grid>
  );
}

export { VotingCard };
