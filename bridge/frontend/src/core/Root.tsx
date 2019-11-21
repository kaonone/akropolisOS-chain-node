import MuiThemeProvider from '@material-ui/styles/ThemeProvider';
import React from 'react';
import { BrowserRouter } from 'react-router-dom';
import Web3 from 'web3';
import { ApiRx, WsProvider } from '@polkadot/api';

import { theme } from 'utils/styles';
import { App } from 'app/App';
import { Api, ApiContext } from 'services/api';
import { ApolloProvider } from 'services/apollo';
import { LocalStorage } from 'services/storage';
import { SUBSTRATE_NODE_URL, SUBSTRATE_NODE_CUSTOM_TYPES } from 'env';
import { ErrorBoundary, CssBaseline } from 'components';

export function Root(): React.ReactElement<{}> {
  // Detect if Web3 is found, if not, ask the user to install Metamask
  if (window.web3) {
    // TODO need to change Web3 instantiating, window.web3 will become deprecated in December 2019
    const web3 = new Web3(window.web3.currentProvider);

    const storage = new LocalStorage('v1');
    const nodeUrl: string | null = storage.get('nodeUrl');

    const provider = new WsProvider(nodeUrl || SUBSTRATE_NODE_URL);

    const substrateApi = ApiRx.create({
      provider,
      types: SUBSTRATE_NODE_CUSTOM_TYPES,
    });

    const api = new Api(web3, substrateApi, storage, provider);

    return (
      <ErrorBoundary>
        <BrowserRouter>
          <ApolloProvider>
            <MuiThemeProvider theme={theme}>
              <ApiContext.Provider value={api}>
                <CssBaseline />
                <App />
              </ApiContext.Provider>
            </MuiThemeProvider>
          </ApolloProvider>
        </BrowserRouter>
      </ErrorBoundary>
    );
  }
  return <div>You need to install Metamask</div>;
}
