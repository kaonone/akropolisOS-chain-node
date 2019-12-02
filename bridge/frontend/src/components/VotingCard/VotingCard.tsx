/* eslint-disable @typescript-eslint/no-unused-vars */
import * as React from 'react';
import Grid from '@material-ui/core/Grid';
import Typography from '@material-ui/core/Typography';
import ExpansionPanel from '@material-ui/core/ExpansionPanel';
import ExpansionPanelSummary from '@material-ui/core/ExpansionPanelSummary';
import ExpansionPanelDetails from '@material-ui/core/ExpansionPanelDetails';

import { useTranslate, tKeys as tKeysAll, ITranslateKey } from 'services/i18n';
import { ShortAddress } from 'components/ShortAddress/ShortAddress';
import { Loading } from 'components/Loading';
import { ContainedCircleArrow, OutlinedCircleArrow } from 'components/icons';
import { ProposalStatus, useLastValidatorsListMessageQuery } from 'generated/bridge-graphql';
import { attachStaticFields } from 'utils/object';
import { filterChildrenByComponent } from 'utils/react';

import { useStyles } from './VotingCard.style';
import { Column } from './Column/Column';

const tKeys = tKeysAll.components.votingCard;

interface IOwnProps {
  children?: React.ReactNode;
  ethBlockNumber: string;
  ethAddress: string;
  status: ProposalStatus;
  expansionPanelTitle: ITranslateKey;
  expansionPanelDetails: React.ReactNode;
}

interface IResultProps {
  children?: React.ReactNode;
}

interface IVotingProps {
  children?: React.ReactNode;
}

function VotingCardComponent(props: IOwnProps) {
  const {
    ethBlockNumber,
    ethAddress,
    status,
    children,
    expansionPanelTitle,
    expansionPanelDetails,
  } = props;

  const classes = useStyles();
  const { t } = useTranslate();
  const [expanded, setExpanded] = React.useState(false);

  const [votingResult] = filterChildrenByComponent<IResultProps>(children, Result);
  const [votingContent] = filterChildrenByComponent<IVotingProps>(children, Voting);

  const isOver = status !== 'PENDING';

  const lastValidatorsListMessage = useLastValidatorsListMessageQuery();
  const neededVotes: number = Number(
    lastValidatorsListMessage.data?.validatorsListMessages[0]?.newHowManyValidatorsDecide || 0,
  );

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
          <Column
            xs={4}
            title={t(tKeys.needed.getKey())}
            value={<Loading gqlResults={lastValidatorsListMessage}>{neededVotes}</Loading>}
          />
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
                <Typography className={classes.showButton}>{expansionPanelTitle}</Typography>
              </ExpansionPanelSummary>
              <ExpansionPanelDetails>{expansionPanelDetails}</ExpansionPanelDetails>
            </ExpansionPanel>
          </Grid>
        </Grid>
      </Grid>
      <Grid item xs={3} className={classes.voting}>
        {isOver ? (
          <Grid container spacing={3} justify="center" direction="column">
            <Grid item>
              <Grid container wrap="nowrap" alignItems="center" justify="center">
                {votingResult.props.children}
              </Grid>
            </Grid>
          </Grid>
        ) : (
          <>{votingContent.props.children}</>
        )}
      </Grid>
    </Grid>
  );
}

function Result(_props: IResultProps) {
  return <noscript />;
}

function Voting(_props: IVotingProps) {
  return <noscript />;
}

export const VotingCard = attachStaticFields(VotingCardComponent, { Result, Voting });
