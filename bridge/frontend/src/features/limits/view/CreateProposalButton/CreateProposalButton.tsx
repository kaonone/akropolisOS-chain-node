import React from 'react';

import { ModalButton } from 'components';
import { useTranslate, tKeys as tKeysAll } from 'services/i18n';

import { LimitsChangingForm } from '../LimitsChangingForm/LimitsChangingForm';

const tKeys = tKeysAll.features.limits;

function CreateProposalButton() {
  const { t } = useTranslate();

  return (
    <ModalButton color="primary" variant="contained" content={t(tKeys.createProposal.getKey())}>
      {({ closeModal }) => <LimitsChangingForm onCancel={closeModal} />}
    </ModalButton>
  );
}

export { CreateProposalButton };
