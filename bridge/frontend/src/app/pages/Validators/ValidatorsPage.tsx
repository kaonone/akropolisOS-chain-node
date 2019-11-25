import * as React from 'react';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { Typography, Hint } from 'components';

const tKeys = tKeysAll.app.pages.validators;

export function ValidatorsPage() {
  const { t } = useTranslate();

  return (
    <>
      <Typography variant="h4" noWrap gutterBottom>
        {t(tKeys.title.getKey())}
      </Typography>
      <Hint>Coming soon</Hint>
    </>
  );
}
