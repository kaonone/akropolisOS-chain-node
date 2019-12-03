import React, { useEffect } from 'react';

import { Button, CircularProgress } from 'components';
import { useApi } from 'services/api';
import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { useCommunication } from 'utils/react';
import { getErrorMsg } from 'utils/getErrorMsg';

interface IProps {
  proposalId: string;
  fromAddress: string;
  disabled: boolean;
}

const tKeys = tKeysAll.features.limits.limitsProposalsList;

function VoteButton(props: IProps) {
  const { proposalId, fromAddress, disabled } = props;
  const api = useApi();
  const { t } = useTranslate();

  const approving = useCommunication(() => api.approveNewLimit(proposalId, fromAddress), [
    proposalId,
    fromAddress,
  ]);

  useEffect(() => {
    approving.error && console.error(getErrorMsg(approving.error));
  }, [approving.error]);

  return (
    <Button
      onClick={approving.execute}
      disabled={disabled || approving.status === 'pending'}
      variant="contained"
      color="primary"
      fullWidth
    >
      {t(tKeys.approve.getKey())}
      {approving.status === 'pending' && <CircularProgress size={24} />}
    </Button>
  );
}

export { VoteButton };
