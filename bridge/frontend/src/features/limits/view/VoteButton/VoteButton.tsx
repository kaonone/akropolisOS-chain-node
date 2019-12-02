import React, { useCallback } from 'react';

import { Button } from 'components';
import { useApi } from 'services/api';
import { useTranslate, tKeys as tKeysAll } from 'services/i18n';

interface IProps {
  proposalId: string;
  fromAddress: string;
}

const tKeys = tKeysAll.features.limits.limitsProposalsList;

function VoteButton(props: IProps) {
  const { proposalId, fromAddress } = props;
  const api = useApi();
  const { t } = useTranslate();

  const handleButtonClick = useCallback(async () => {
    try {
      await api.approveNewLimit(proposalId, fromAddress);
    } catch (error) {
      throw new Error(error);
    }
  }, [proposalId, fromAddress]);

  return (
    <Button onClick={handleButtonClick} variant="contained" color="primary" fullWidth>
      {t(tKeys.approve.getKey())}
    </Button>
  );
}

export { VoteButton };
