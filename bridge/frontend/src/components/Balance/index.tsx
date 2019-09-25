import React from "react";
import BN from 'bn.js';
import { Typography, LinearProgress, Box } from '@material-ui/core';

import { useSubscribable } from '~util/hooks';
import { formatDai } from '~util/formatDai';
import { useApi } from "../context";

interface IProps {
  type: 'ethereum' | 'substrate';
  address: string;
  name?: string;
}

export function Balance({ address, type, name }: IProps) {
  const api = useApi();
  const [balance, { error, loaded }] = useSubscribable(
    type === 'ethereum'
      ? () => api.getEthBalance$(address)
      : () => api.getSubstrateBalance$(address),
    [address],
    new BN(0),
  );

  return (
    <>
      {!!name && <Typography variant="h5">{name}</Typography>}
      <Typography>Address: {address}</Typography>
      <Typography>Balance:
        {!loaded && !error && <Box display="inline"><LinearProgress /></Box>}
        {!!error && <Typography component="span" color="error">{error}</Typography>}
        {loaded && !error && ` ${formatDai(balance)} DAI`}
      </Typography>
    </>
  );
}
