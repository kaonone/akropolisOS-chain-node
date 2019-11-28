import * as React from 'react';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { MakeTableType } from 'components/Table/Table';
import { Typography, Hint, Table as GeneralTable, Loading, Grid } from 'components';
import { useLimitsQuery, Limit, LimitKind } from 'generated/bridge-graphql';

import { KindCell, ValueCell, EthBlockNumberCell } from './LimitsTableCells';

const Table = GeneralTable as MakeTableType<Limit>;

const tKeys = tKeysAll.features.settings.limits;

const limitsNames: Record<LimitKind, string> = {
  MIN_HOST_TRANSACTION_VALUE: 'Min host transaction value',
  MAX_HOST_TRANSACTION_VALUE: 'Max host transaction value',
  DAY_HOST_MAX_LIMIT: 'Day host max limit',
  DAY_HOST_MAX_LIMIT_FOR_ONE_ADDRESS: 'Day host max limit for one address',
  MAX_HOST_PENDING_TRANSACTION_LIMIT: 'Max host pending transaction limit',
  MIN_GUEST_TRANSACTION_VALUE: 'Min guest transaction value',
  MAX_GUEST_TRANSACTION_VALUE: 'Max guest transaction value',
  DAY_GUEST_MAX_LIMIT: 'Day guest max limit',
  DAY_GUEST_MAX_LIMIT_FOR_ONE_ADDRESS: 'Day guest max limit for one address',
  MAX_GUEST_PENDING_TRANSACTION_LIMIT: 'Max guest pending transaction limit',
};

interface IProps {
  isCompactStyle?: boolean;
}

export function LimitsList(props: IProps) {
  const { isCompactStyle } = props;
  const { t } = useTranslate();

  const { loading, data: limitsData, error } = useLimitsQuery();

  const limits = limitsData?.limits;

  return (
    <Loading meta={{ loaded: !loading, error: error && error.message }}>
      {!limits || !limits.length ? (
        <Hint>
          <Typography>{t(tKeys.notFound.getKey())}</Typography>
        </Hint>
      ) : (
        <Table data={limits} isCompactStyle={isCompactStyle}>
          <Table.Column>
            <Table.Head>{t(tKeys.kind.getKey())}</Table.Head>
            <Table.Cell>{({ data }) => <KindCell kind={limitsNames[data.kind]} />}</Table.Cell>
          </Table.Column>
          <Table.Column>
            <Table.Head align="center">{t(tKeys.value.getKey())}</Table.Head>
            <Table.Cell align="center">{({ data }) => <ValueCell value={data.value} />}</Table.Cell>
          </Table.Column>
          <Table.Column>
            <Table.Head align="center">{t(tKeys.ethBlockNumber.getKey())}</Table.Head>
            <Table.Cell align="center">
              {({ data }) => <EthBlockNumberCell blockNumber={data.ethBlockNumber} />}
            </Table.Cell>
          </Table.Column>
        </Table>
      )}
    </Loading>
  );
}
