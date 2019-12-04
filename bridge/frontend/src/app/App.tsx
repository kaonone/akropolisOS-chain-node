import * as React from 'react';
import { Switch, Route, Redirect } from 'react-router';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';

import { BaseLayout } from './components/BaseLayout/BaseLayout';
import { DemoPage, BridgePage, SettingsPage, LimitsPage, ValidatorsPage } from './pages';
import { routes } from './routes';

const tKeys = tKeysAll.app;

export function App() {
  const { t } = useTranslate();

  return (
    <BaseLayout title={t(tKeys.mainTitle.getKey())}>
      <Switch>
        {process.env.NODE_ENV !== 'production' && (
          <Route exact path={routes.demo.getRoutePath()} component={DemoPage} />
        )}
        <Route exact path={routes.bridge.sourceChain.getRoutePath()} component={BridgePage} />
        <Route exact path={routes.limits.getRoutePath()} component={LimitsPage} />
        <Route exact path={routes.validators.getRoutePath()} component={ValidatorsPage} />
        <Route exact path={routes.settings.getRoutePath()} component={SettingsPage} />
        <Redirect to={routes.bridge.sourceChain.getRedirectPath({ sourceChain: 'ethereum' })} />
      </Switch>
    </BaseLayout>
  );
}
