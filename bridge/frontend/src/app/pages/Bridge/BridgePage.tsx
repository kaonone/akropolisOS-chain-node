import React from 'react';
import SwipeableViews from 'react-swipeable-views';
import { makeStyles } from '@material-ui/core/styles';
import { RouteComponentProps } from 'react-router';
import { Link } from 'react-router-dom';

import { Grid, Typography, Paper, Tabs, Tab, Box } from 'components';
import { EthereumToSubstrate, SubstrateToEthereum } from 'features/tokenTransfer';
import { Messages } from 'features/transfersHistory';

import { routes } from '../../routes';

const useStyles = makeStyles(theme => ({
  root: {
    padding: theme.spacing(3),
    maxWidth: 1200,
    margin: '0 auto',
  },
}));

type SourceChain = 'ethereum' | 'substrate' | 'settings';

const viewIndexBySourceChain: Record<SourceChain, number> = {
  ethereum: 0,
  substrate: 1,
  settings: 2,
};

function BridgePage(props: RouteComponentProps<{ sourceChain: SourceChain }>) {
  const classes = useStyles();

  const { match } = props;
  const { sourceChain } = match.params;

  const currentTabIndex = viewIndexBySourceChain[sourceChain] || 0;

  return (
    <Grid container spacing={3} className={classes.root}>
      <Grid item xs={12}>
        <Typography variant="h2" align="center" gutterBottom>
          Ethereum DAI {'<-->'} AkropolisOS Bridge
        </Typography>
      </Grid>
      <Grid item xs={12}>
        <Paper>
          <Tabs
            value={currentTabIndex}
            indicatorColor="primary"
            textColor="primary"
            variant="fullWidth"
          >
            <Tab
              label="Ethereum to Substrate"
              component={Link}
              to={routes.sourceChain.getRedirectPath({ sourceChain: 'ethereum' })}
            />
            <Tab
              label="Substrate to Ethereum"
              component={Link}
              to={routes.sourceChain.getRedirectPath({ sourceChain: 'substrate' })}
            />
            <Tab
              label="Settings"
              component={Link}
              to={routes.sourceChain.getRedirectPath({ sourceChain: 'settings' })}
            />
          </Tabs>
        </Paper>
        <SwipeableViews index={currentTabIndex}>
          <Box p={2}>
            <EthereumToSubstrate />
          </Box>
          <Box p={2}>
            <SubstrateToEthereum />
          </Box>
          <Box p={2}>
            <Settings />
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
