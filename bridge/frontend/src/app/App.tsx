import * as React from 'react';
import { Switch, Route, Redirect } from 'react-router';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';

import { BaseLayout } from './components/BaseLayout/BaseLayout';
import { DemoPage } from './pages/Demo/DemoPage';
import { BridgePage } from './pages/Bridge/BridgePage';
import { routes } from './routes';

const tKeys = tKeysAll.app;

export function App() {
  const { t } = useTranslate();

  return (
    <Switch>
      <BaseLayout title={t(tKeys.mainTitle.getKey())}>
        {process.env.NODE_ENV !== 'production' && (
          <Route exact path={routes.demo.getRoutePath()} component={DemoPage} />
        )}
        <Route exact path={routes.sourceChain.getRoutePath()} component={BridgePage} />
        <Redirect to={routes.sourceChain.getRedirectPath({ sourceChain: 'ethereum' })} />
      </BaseLayout>
    </Switch>
  );
}
