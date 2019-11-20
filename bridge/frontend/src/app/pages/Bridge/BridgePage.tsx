import React from 'react';
import SwipeableViews from 'react-swipeable-views';
import { makeStyles } from '@material-ui/core/styles';
import { RouteComponentProps } from 'react-router';
import { Link } from 'react-router-dom';

import { Grid, Typography, Paper, Tabs, Tab, Box, TransfersList, Loading } from 'components';
import { EthereumToSubstrate, SubstrateToEthereum } from 'features/tokenTransfer';
import { useApi } from 'services/api';
import { useSubscribable } from 'utils/hooks';
import { useMessagesByIdsQuery } from 'generated/bridge-graphql';

import { routes } from '../../routes';

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

// subgraph throws an error if the identifiers are empty
const mockIds = ['0x0000000000000000000000000000000000000000000000000000000000000000'];

function BridgePage(props: RouteComponentProps<{ sourceChain: SourceChain }>) {
  const classes = useStyles();

  const { sourceChain } = props.match.params;

  const currentTabIndex = viewIndexBySourceChain[sourceChain] || 0;

  const api = useApi();
  const [ids, idsMeta] = useSubscribable(() => api.getTransactions$(), [], []);

  const { loading, data, error } = useMessagesByIdsQuery({
    variables: { ids: ids && ids.length ? ids : mockIds },
  });
  const messages = (ids.length && data?.messages) || null;

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
          </Tabs>
        </Paper>
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
        <Loading meta={[idsMeta, { loaded: !loading, error: error && error.message }]}>
          {messages && <TransfersList messages={messages} />}
        </Loading>
      </Grid>
    </Grid>
  );
}

export { BridgePage };
