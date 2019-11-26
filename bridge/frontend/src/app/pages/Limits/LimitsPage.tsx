import * as React from 'react';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { Typography } from 'components';
import { LimitsList } from 'features/settings/LimitsList/LimitsList';

const tKeys = tKeysAll.app.pages.limits;

export function LimitsPage() {
  const { t } = useTranslate();

  return (
    <>
      <Typography variant="h4" noWrap gutterBottom>
        {t(tKeys.title.getKey())}
      </Typography>
      <LimitsList />
    </>
  );
}
