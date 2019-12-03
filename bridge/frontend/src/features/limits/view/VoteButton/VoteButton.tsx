import React, { useCallback } from 'react';
import { ButtonProps } from '@material-ui/core/Button';

import { Button } from 'components';
import { useApi } from 'services/api';
import { useTranslate, tKeys as tKeysAll } from 'services/i18n';

interface IProps {
  proposalId: string;
  fromAddress: string;
}

const tKeys = tKeysAll.features.limits.limitsProposalsList;

function VoteButton(props: IProps & ButtonProps) {
  const { proposalId, fromAddress, disabled } = props;
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
    <Button
      onClick={handleButtonClick}
      variant="contained"
      color="primary"
      disabled={disabled}
      fullWidth
    >
      {t(tKeys.approve.getKey())}
    </Button>
  );
}

export { VoteButton };
