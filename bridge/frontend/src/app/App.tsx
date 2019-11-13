import * as React from 'react';
import { Switch, Route, Redirect } from 'react-router';

import { DemoPage } from './pages/Demo/DemoPage';
import { BridgePage } from './pages/Bridge/BridgePage';
import { routes } from './routes';

export function App() {
  return (
    <Switch>
      {process.env.NODE_ENV !== 'production' && (
        <Route exact path={routes.demo.getRoutePath()} component={DemoPage} />
      )}
      <Route exact path={routes.sourceChain.getRoutePath()} component={BridgePage} />
      <Redirect to={routes.sourceChain.getRedirectPath({ sourceChain: 'ethereum' })} />
    </Switch>
  );
}
