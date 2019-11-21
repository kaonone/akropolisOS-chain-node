import * as React from 'react';
import ForwardIcon from '@material-ui/icons/Forward';
import Grid from '@material-ui/core/Grid';
import Avatar from '@material-ui/core/Avatar';
import BaseIdentityIcon from '@polkadot/react-identicon';
import { encodeAddress } from '@polkadot/util-crypto';
import Jazzicon, { jsNumberForAddress } from 'react-jazzicon';

import { ShortAddress, TransactionStatus } from 'components';
import { BalanceValue } from 'components/BalanceValue';
import { makeStyles } from 'utils/styles';
import { isHex } from 'utils/hex/isHex';
import { Status, Direction } from 'generated/bridge-graphql';

const useStyles = makeStyles(() => {
  return {
    substrateIcon: {
      width: '100%',
      height: '100%',
      cursor: 'default',
      '& svg': {
        width: '100%',
        height: '100%',
      },
    },
  };
});

export function AddressCell({ address, isSubstrate }: { address: string; isSubstrate: boolean }) {
  const classes = useStyles();

  return (
    <Grid container alignItems="center" justify="center" spacing={1}>
      <Grid item>
        <Avatar>
          {isSubstrate ? (
            <BaseIdentityIcon className={classes.substrateIcon} value={address} />
          ) : (
            <Jazzicon diameter={40} seed={jsNumberForAddress(address)} />
          )}
        </Avatar>
      </Grid>
      <Grid item>
        <ShortAddress address={isSubstrate && isHex(address) ? encodeAddress(address) : address} />
      </Grid>
    </Grid>
  );
}

export function AmountCell({ amount }: { amount: string }) {
  return <BalanceValue input={amount} />;
}

export function StatusCell({ status }: { status: Status }) {
  return <TransactionStatus status={status} />;
}

export function DirectionCell({ direction }: { direction: Direction }) {
  return (
    <Grid container alignItems="center" justify="center" spacing={1}>
      {direction === 'ETH2SUB' ? (
        <>
          <Grid item>ETH</Grid>
          <ForwardIcon color="primary" />
          <Grid item>SUB</Grid>
        </>
      ) : (
        <>
          <Grid item>SUB</Grid>
          <ForwardIcon color="primary" />
          <Grid item>ETH</Grid>
        </>
      )}
    </Grid>
  );
}

export function BlockNumberCell({ blockNumber }: { blockNumber: string }) {
  return <>{blockNumber}</>;
}
