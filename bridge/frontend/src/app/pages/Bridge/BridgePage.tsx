import React from 'react';
import SwipeableViews from 'react-swipeable-views';
import { makeStyles } from '@material-ui/core/styles';
import { RouteComponentProps } from 'react-router';
import { Link } from 'react-router-dom';

import { Grid, Typography, Paper, Tabs, Tab, Box } from 'components';
import { EthereumToSubstrate, SubstrateToEthereum } from 'features/tokenTransfer';

const useStyles = makeStyles(theme => ({
  root: {
    padding: theme.spacing(3),
    maxWidth: 1200,
    margin: '0 auto',
  },
}));

function BridgePage(props: RouteComponentProps<{ sourceChain: 'ethereum' | 'substrate' }>) {
  const classes = useStyles();

  const {
    match: {
      params: { sourceChain },
    },
  } = props;

  const viewIndexBySourceChain: Record<'ethereum' | 'substrate', number> = {
    ethereum: 0,
    substrate: 1,
  };

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
            value={viewIndexBySourceChain[sourceChain] || 0}
            indicatorColor="primary"
            textColor="primary"
            variant="fullWidth"
          >
            <Tab label="Ethereum to Substrate" component={Link} to="/ethereum" />
            <Tab label="Substrate to Ethereum" component={Link} to="/substrate" />
          </Tabs>
          <SwipeableViews index={viewIndexBySourceChain[sourceChain] || 0}>
            <Box p={2}>
              <EthereumToSubstrate />
            </Box>
            <Box p={2}>
              <SubstrateToEthereum />
            </Box>
          </SwipeableViews>
        </Paper>
      </Grid>
    </Grid>
  );
}

export { BridgePage };
