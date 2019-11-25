import React from 'react';
import SwipeableViews from 'react-swipeable-views';
import { makeStyles } from '@material-ui/core/styles';
import { RouteComponentProps } from 'react-router';

import { Grid, Box } from 'components';
import { EthereumToSubstrate, SubstrateToEthereum } from 'features/tokenTransfer';
import { Messages } from 'features/transfersHistory';

const useStyles = makeStyles(theme => ({
  root: {
    padding: theme.spacing(3),
    maxWidth: 1200,
    margin: '0 auto',
  },
}));

type SourceChain = 'ethereum' | 'substrate';

const viewIndexBySourceChain: Record<SourceChain, number> = {
  ethereum: 0,
  substrate: 1,
};

function BridgePage(props: RouteComponentProps<{ sourceChain: SourceChain }>) {
  const classes = useStyles();

  const { match } = props;
  const { sourceChain } = match.params;

  const currentTabIndex = viewIndexBySourceChain[sourceChain] || 0;

  return (
    <Grid container spacing={3} className={classes.root}>
      <Grid item xs={12}>
        <SwipeableViews index={currentTabIndex}>
          <Box p={2}>
            <EthereumToSubstrate />
          </Box>
          <Box p={2}>
            <SubstrateToEthereum />
          </Box>
        </SwipeableViews>
      </Grid>
      <Grid item xs={12}>
        <Messages />
      </Grid>
    </Grid>
  );
}

export { BridgePage };
