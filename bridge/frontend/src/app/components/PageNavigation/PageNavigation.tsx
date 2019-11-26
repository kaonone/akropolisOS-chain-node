import * as React from 'react';
import { Route, RouteComponentProps } from 'react-router';
import { Link } from 'react-router-dom';

import { routes } from 'app/routes';
import { Tabs, Tab } from 'components';

function PageNavigation() {
  return (
    <Route path="/:page">
      {({ match }: RouteComponentProps<{ page: string }>) => (
        <Tabs
          value={(match && match.params.page) || 'ethereum'}
          indicatorColor="primary"
          textColor="primary"
        >
          <Tab
            label="Bridge"
            component={Link}
            value={routes.bridge.getElementKey()}
            to={routes.bridge.sourceChain.getRedirectPath({ sourceChain: 'ethereum' })}
          />
          <Tab
            label="Limits"
            component={Link}
            value={routes.limits.getElementKey()}
            to={routes.limits.getRedirectPath()}
          />
          <Tab
            label="Validators"
            component={Link}
            value={routes.validators.getElementKey()}
            to={routes.validators.getRedirectPath()}
          />
          <Tab
            label="Settings"
            component={Link}
            value={routes.settings.getElementKey()}
            to={routes.settings.getRedirectPath()}
          />
        </Tabs>
      )}
    </Route>
  );
}

export { PageNavigation };
