import React from 'react';
import * as colors from '@material-ui/core/colors';
import Chip from '@material-ui/core/Chip';

export type Status =
  | 'PENDING'
  | 'WITHDRAW'
  | 'APPROVED'
  | 'CANCELED'
  | 'CONFIRMED'
  | 'CONFIRMED_WITHDRAW';

interface IProps {
  status: Status;
}

function TransactionStatus(props: IProps) {
  const { status } = props;

  const backgrounds = {
    PENDING: colors.blue[500],
    WITHDRAW: colors.purple[500],
    APPROVED: colors.teal[500],
    CANCELED: colors.red[500],
    CONFIRMED: colors.lightGreen[500],
    CONFIRMED_WITHDRAW: colors.indigo[500],
  };

  const statuses = {
    PENDING: 'pending',
    WITHDRAW: 'withdraw',
    APPROVED: 'approved',
    CANCELED: 'canceled',
    CONFIRMED: 'confirmed',
    CONFIRMED_WITHDRAW: 'confirmed withdraw',
  };

  return (
    <Chip
      style={{ background: backgrounds[status], color: '#fff' }}
      label={statuses[status].toUpperCase()}
    />
  );
}

export { TransactionStatus };
