import * as React from 'react';
import { Route } from 'react-router';
import { Link } from 'react-router-dom';

import { routes } from 'app/routes';
import { Tabs, Tab } from 'components';

type SourceChain = 'ethereum' | 'substrate' | 'settings';

const viewIndexBySourceChain: Record<SourceChain, number> = {
  ethereum: 0,
  substrate: 1,
  settings: 2,
};

function PageNavigation() {
  return (
    <Route path={routes.sourceChain.getRoutePath()}>
      {({ match }) => (
        <Tabs
          value={
            (match?.params.sourceChain &&
              viewIndexBySourceChain[match.params.sourceChain as SourceChain]) ||
            0
          }
          indicatorColor="primary"
          textColor="primary"
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
      )}
    </Route>
  );
}

export { PageNavigation };
