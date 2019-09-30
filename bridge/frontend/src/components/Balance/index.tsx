import React from "react";
import BN from 'bn.js';
import { Typography, LinearProgress, Box } from '@material-ui/core';

import { DEFAULT_DECIMALS } from '~env';
import { useSubscribable } from '~util/hooks';
import { fromBaseUnit } from '~util/fromBaseUnit';
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
      <Typography component="div">Balance:
        {!loaded && !error && <Box display="inline"><LinearProgress /></Box>}
        {!!error && <Typography component="span" color="error">{error}</Typography>}
        {loaded && !error && ` ${fromBaseUnit(balance, DEFAULT_DECIMALS)} DAI`}
      </Typography>
    </>
  );
}
