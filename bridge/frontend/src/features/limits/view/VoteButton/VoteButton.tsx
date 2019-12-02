import React, { useCallback } from 'react';

import { Button } from 'components';
import { useApi } from 'services/api';

function VoteButton() {
  const api = useApi();

  const promise = useCallback(() => api.approveNewLimit(), []);

  const handleButtonClick = useCallback(() => {}, []);

  return (
    <Button onClick={handleButtonClick} variant="contained" color="primary" fullWidth>
      Approve
    </Button>
  );
}

export { VoteButton };
