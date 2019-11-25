import * as React from 'react';

import { LocalSettings } from 'features/settings';
import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { Typography, Grid } from 'components';

const tKeys = tKeysAll.app.pages.settings;

export function SettingsPage() {
  const { t } = useTranslate();

  return (
    <Grid container spacing={4} justify="center">
      <Grid item xs={6}>
        <Typography variant="h4" noWrap gutterBottom>
          {t(tKeys.title.getKey())}
        </Typography>
        <LocalSettings />
      </Grid>
    </Grid>
  );
}
