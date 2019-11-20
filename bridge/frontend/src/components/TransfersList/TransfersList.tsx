import * as React from 'react';
import Typography from '@material-ui/core/Typography';
import Grid from '@material-ui/core/Grid';

import { Table as GeneralTable, MakeTableType } from 'components/Table/Table';
import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { Message } from 'generated/bridge-graphql';

import {
  AddressCell,
  AmountCell,
  StatusCell,
  DirectionCell,
  BlockNumberCell,
} from './TransfersTableCells';

const Table = GeneralTable as MakeTableType<Message>;

interface IProps {
  messages: Message[];
}

const tKeys = tKeysAll.components.transfersList;

function TransfersList(props: IProps) {
  const { messages } = props;
  const { t } = useTranslate();

  return (
    <Grid container spacing={2}>
      <Grid item xs={12}>
        <Typography variant="h4">{t(tKeys.title.getKey())}</Typography>
      </Grid>
      <Grid item xs={12}>
        <Table data={messages} separated>
          <Table.Column>
            <Table.Head align="center">#</Table.Head>
            <Table.Cell align="center">
              {({ index }) => <Typography variant="body1">{index + 1}</Typography>}
            </Table.Cell>
          </Table.Column>
          <Table.Column>
            <Table.Head align="center">{t(tKeys.direction.getKey())}</Table.Head>
            <Table.Cell align="center">
              {({ data }) => <DirectionCell direction={data.direction} />}
            </Table.Cell>
          </Table.Column>
          <Table.Column>
            <Table.Head align="center">{t(tKeys.ethAddress.getKey())}</Table.Head>
            <Table.Cell align="center">
              {({ data }) => (
                <AddressCell isSubstrate={false} address={data.ethAddress} />
              )}
            </Table.Cell>
          </Table.Column>
          <Table.Column>
            <Table.Head align="center">{t(tKeys.subAddress.getKey())}</Table.Head>
            <Table.Cell align="center">
              {({ data }) => (
                <AddressCell isSubstrate address={data.subAddress} />
              )}
            </Table.Cell>
          </Table.Column>
          <Table.Column>
            <Table.Head>{t(tKeys.amount.getKey())}</Table.Head>
            <Table.Cell>
              {({ data }) => <AmountCell amount={data.amount} />}
            </Table.Cell>
          </Table.Column>
          <Table.Column>
            <Table.Head align="center">{t(tKeys.status.getKey())}</Table.Head>
            <Table.Cell align="center">
              {({ data }) => <StatusCell status={data.status} />}
            </Table.Cell>
          </Table.Column>
          <Table.Column>
            <Table.Head>{t(tKeys.blockNumber.getKey())}</Table.Head>
            <Table.Cell>
              {({ data }) => (
                <BlockNumberCell blockNumber={data.ethBlockNumber || ''} />
              )}
            </Table.Cell>
          </Table.Column>
        </Table>
      </Grid>
    </Grid>
  );
}

export { TransfersList };
