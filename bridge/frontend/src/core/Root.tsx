import MuiThemeProvider from '@material-ui/styles/ThemeProvider';
import React from 'react';
import { BrowserRouter } from 'react-router-dom';
import Web3 from 'web3';
import { ApiRx, WsProvider } from '@polkadot/api';

import { theme } from 'utils/styles';
import { App } from 'app/App';
import { Api, ApiContext } from 'services/api';
import { SUBSTRATE_NODE_URL, SUBSTRATE_NODE_CUSTOM_TYPES } from 'env';
import { ErrorBoundary, CssBaseline } from 'components';

export function Root(): React.ReactElement<{}> {
  // Detect if Web3 is found, if not, ask the user to install Metamask
  if (window.web3) { // TODO need to change Web3 instantiating, window.web3 will become deprecated in December 2019
    const web3 = new Web3(window.web3.currentProvider);
    const substrateApi = ApiRx.create({
      provider: new WsProvider(SUBSTRATE_NODE_URL),
      types: SUBSTRATE_NODE_CUSTOM_TYPES,
    });
    const api = new Api(web3, substrateApi);

    return (
      <ErrorBoundary>
        <BrowserRouter>
          <MuiThemeProvider theme={theme}>
            <ApiContext.Provider value={api}>
              <CssBaseline />
              <App />
            </ApiContext.Provider>
          </MuiThemeProvider>
        </BrowserRouter>
      </ErrorBoundary>
    );
  }
  return <div>You need to install Metamask</div>;
}
