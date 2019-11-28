import * as React from 'react';
import Grid from '@material-ui/core/Grid';
import Typography from '@material-ui/core/Typography';
import ExpansionPanel from '@material-ui/core/ExpansionPanel';
import ExpansionPanelSummary from '@material-ui/core/ExpansionPanelSummary';
import ExpansionPanelDetails from '@material-ui/core/ExpansionPanelDetails';
import Button from '@material-ui/core/Button';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { ShortAddress } from 'components/ShortAddress/ShortAddress';
import { ContainedCircleArrow, OutlinedCircleArrow } from 'components/icons';
import { LimitsList } from 'features/settings';
import { useLimitProposalQuery } from 'generated/bridge-graphql';

import { VotingResult } from './VotingResult/VotingResult';
import { useStyles } from './VotingCard.style';
import { Column } from './Column/Column';

const tKeys = tKeysAll.components.votingCard;

export type VotingStatus = 'PENDING' | 'APPROVED' | 'DECLINED';

interface IOwnProps {
  id: string;
  needVoted: number;
  showLimitsList: boolean;
}

function VotingCard(props: IOwnProps) {
  const { id, needVoted, showLimitsList } = props;

  const classes = useStyles();
  const { t } = useTranslate();
  const [expanded, setExpanded] = React.useState(false);

  const { loading, data, error } = useLimitProposalQuery({ variables: { id } });

  const limitProposal = data?.limitProposal;
  const ethBlockNumber = limitProposal?.ethBlockNumber || '';
  const fromAddress = limitProposal?.ethAddress || '';
  const votingStatus = limitProposal?.status || 'PENDING';

  const isOver = votingStatus !== 'PENDING';

  const handleExpansionPanelChange = (_event: React.ChangeEvent<{}>, isExpanded: boolean) => {
    setExpanded(isExpanded);
  };

  return (
    <Grid className={classes.root} container wrap="nowrap">
      <Grid item xs={9} className={classes.mainInformation}>
        <Grid container spacing={3}>
          <Column xs={4} title="Block number" value={ethBlockNumber} />
          <Column
            xs={4}
            title="from"
            value={<ShortAddress className={classes.address} address={fromAddress} />}
          />
          <Column xs={4} title={t(tKeys.needed.getKey())} value={needVoted} />
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
                  <LimitsList isCompactStyle />
                </ExpansionPanelDetails>
              </ExpansionPanel>
            </Grid>
          )}
        </Grid>
      </Grid>
      {!isOver && (
        <Grid item xs={3} className={classes.voting}>
          <Button variant="contained" color="primary" disabled fullWidth>
            Approve
          </Button>
        </Grid>
      )}
      {isOver && (
        <Grid item xs={3} className={classes.votingResult}>
          <VotingResult votingStatus={votingStatus} />
        </Grid>
      )}
    </Grid>
  );
}

export { VotingCard };
