import React from 'react';

import { ModalButton } from 'components';
import { useTranslate, tKeys as tKeysAll } from 'services/i18n';

import { LimitsChangingForm } from '../LimitsChangingForm/LimitsChangingForm';

interface IProps {
  canVote: boolean;
}

const tKeys = tKeysAll.features.limits;

function CreateProposalButton(props: IProps) {
  const { canVote } = props;
  const { t } = useTranslate();

  return (
    <ModalButton
      color="primary"
      variant="contained"
      content={t(tKeys.createProposal.getKey())}
      disabled={!canVote}
    >
      {({ closeModal }) => <LimitsChangingForm onCancel={closeModal} />}
    </ModalButton>
  );
}

export { CreateProposalButton };
