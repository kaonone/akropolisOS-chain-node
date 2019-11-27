import * as React from 'react';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { MakeTableType } from 'components/Table/Table';
import { Typography, Hint, Table as GeneralTable, Loading, Grid } from 'components';
import { useLimitsQuery, Limit } from 'generated/bridge-graphql';

const Table = GeneralTable as MakeTableType<Limit>;

const tKeys = tKeysAll.features.settings.limits;

export function LimitsList() {
  const { t } = useTranslate();

  const limitsResult = useLimitsQuery();

  const list = limitsResult.data?.limits;

  return (
    <Grid container spacing={3}>
      <Grid item xs={8}>
        <Loading gqlResults={limitsResult}>
          {!list || !list.length ? (
            <Hint>
              <Typography>{t(tKeys.notFound.getKey())}</Typography>
            </Hint>
          ) : (
            <Table data={list} separated>
              <Table.Column>
                <Table.Head>{t(tKeys.kind.getKey())}</Table.Head>
                <Table.Cell>{({ data }) => t(tKeys.items[data.kind].getKey())}</Table.Cell>
              </Table.Column>
              <Table.Column>
                <Table.Head>{t(tKeys.value.getKey())}</Table.Head>
                <Table.Cell prop="value" />
              </Table.Column>
              <Table.Column>
                <Table.Head>{t(tKeys.ethBlockNumber.getKey())}</Table.Head>
                <Table.Cell prop="ethBlockNumber" />
              </Table.Column>
            </Table>
          )}
        </Loading>
      </Grid>
      <Grid item xs={8} />
    </Grid>
  );
}
