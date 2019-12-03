import * as React from 'react';
import toCamelCase from 'to-camel-case';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { MakeTableType } from 'components/Table/Table';
import { Typography, Hint, Table as GeneralTable, Loading } from 'components';
import { useLimitsQuery, Limit, LimitProposal, LimitKind } from 'generated/bridge-graphql';

const Table = GeneralTable as MakeTableType<Partial<Limit>>;

const tKeys = tKeysAll.features.limits.limitsList;

interface IProps {
  limitProposal?: LimitProposal;
  variant?: React.ComponentProps<typeof Table>['variant'];
}

export function LimitsList(props: IProps) {
  const { limitProposal, variant } = props;
  const { t } = useTranslate();

  const limitsResult = useLimitsQuery();

  const limitsList =
    limitProposal &&
    Object.values(LimitKind).map(kind => ({
      kind,
      value: limitProposal[toCamelCase(kind) as keyof LimitProposal],
      ethBlockNumber: limitProposal.ethBlockNumber,
    }));

  const list = limitsList || limitsResult.data?.limits;

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
            <Table.Cell>{({ data }) => data.kind && t(tKeys.items[data.kind].getKey())}</Table.Cell>
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
