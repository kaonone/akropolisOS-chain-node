import * as React from 'react';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { MakeTableType } from 'components/Table/Table';
import { Typography, Hint, Table as GeneralTable, Loading } from 'components';
import { useLimitsQuery, Limit } from 'generated/bridge-graphql';

const Table = GeneralTable as MakeTableType<Limit>;

const tKeys = tKeysAll.features.limitsList;

interface IProps {
  variant?: React.ComponentProps<typeof Table>['variant'];
}

export function LimitsList(props: IProps) {
  const { variant } = props;
  const { t } = useTranslate();

  const limitsResult = useLimitsQuery();

  const list = limitsResult.data?.limits;

  return (
    <Loading gqlResults={limitsResult}>
      {!list || !list.length ? (
        <Hint>
          <Typography>{t(tKeys.notFound.getKey())}</Typography>
        </Hint>
      ) : (
        <Table data={list} variant={variant}>
          <Table.Column>
            <Table.Head>{t(tKeys.kind.getKey())}</Table.Head>
            <Table.Cell>{({ data }) => t(tKeys.items[data.kind].getKey())}</Table.Cell>
          </Table.Column>
          <Table.Column>
            <Table.Head align="center">{t(tKeys.value.getKey())}</Table.Head>
            <Table.Cell prop="value" align="center" />
          </Table.Column>
          <Table.Column>
            <Table.Head align="center">{t(tKeys.ethBlockNumber.getKey())}</Table.Head>
            <Table.Cell prop="ethBlockNumber" align="center" />
          </Table.Column>
        </Table>
      )}
    </Loading>
  );
}
