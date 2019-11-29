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
import { LimitsList } from 'features/settings/view/LimitsList/LimitsList';
import { LimitProposal } from 'generated/bridge-graphql';

import { VotingResult } from './VotingResult/VotingResult';
import { useStyles } from './VotingCard.style';
import { Column } from './Column/Column';

const tKeys = tKeysAll.components.votingCard;

interface IOwnProps {
  limitProposal: LimitProposal;
  neededVotes: number;
}

function VotingCard(props: IOwnProps) {
  const { limitProposal, neededVotes } = props;

  const classes = useStyles();
  const { t } = useTranslate();
  const [expanded, setExpanded] = React.useState(false);

  const { ethBlockNumber, ethAddress, status } = limitProposal;
  const isOver = status !== 'PENDING';

  const handleExpansionPanelChange = (_event: React.ChangeEvent<{}>, isExpanded: boolean) => {
    setExpanded(isExpanded);
  };

  return (
    <Grid className={classes.root} container wrap="nowrap">
      <Grid item xs={9} className={classes.mainInformation}>
        <Grid container spacing={3}>
          <Column xs={4} title={t(tKeys.blockNumber.getKey())} value={ethBlockNumber} />
          <Column
            xs={4}
            title={t(tKeys.from.getKey())}
            value={<ShortAddress address={ethAddress} />}
          />
          <Column xs={4} title={t(tKeys.needed.getKey())} value={neededVotes} />
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
                <LimitsList variant="compact" />
              </ExpansionPanelDetails>
            </ExpansionPanel>
          </Grid>
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
          <VotingResult votingStatus={status} />
        </Grid>
      )}
    </Grid>
  );
}

export { VotingCard };
