import * as React from 'react';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { LimitsList, LimitsProposalsList } from 'features/settings';
import { Typography } from 'components';

const tKeys = tKeysAll.app.pages.limits;

export function LimitsPage() {
  const { t } = useTranslate();

  return (
    <>
      <Typography variant="h4" noWrap gutterBottom>
        {t(tKeys.title.getKey())}
      </Typography>
      <LimitsList />
      <LimitsProposalsList />
    </>
  );
}
