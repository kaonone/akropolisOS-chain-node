import React from 'react';

import { useApi } from 'services/api';
import { useSubscribable } from 'utils/hooks';
import { useMessagesByIdsQuery, Message } from 'generated/bridge-graphql';
import { TransfersList } from 'features/transfersHistory/view/TransfersList/TransfersList';
import { Loading, Typography, Hint } from 'components';
import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { usePagination } from 'utils/hooks/usePagination';

// subgraph throws an error if the identifiers are empty
const mockIds = ['0x0000000000000000000000000000000000000000000000000000000000000000'];

function Messages() {
  const { t } = useTranslate();
  const tKeys = tKeysAll.features.transfersList;

  const api = useApi();
  const [transactions, transactionsMeta] = useSubscribable(() => api.getTransactions$(), [], []);

  const ids = transactions.map(transaction => transaction.id) || [];
  const { items: paginatedIds, paginationView } = usePagination(ids);

  const { loading, data, error } = useMessagesByIdsQuery({
    variables: { ids: (paginatedIds.length && paginatedIds) || mockIds },
  });

  const messages =
    (transactions.length &&
      ids.map(id => {
        return (
          (data?.messages && data.messages.find(item => item.id === id)) ||
          transactions.find(item => item.id === id)
        );
      })) ||
    [];

  return (
    <Loading meta={[transactionsMeta, { loaded: !loading, error: error && error.message }]}>
      {!paginatedIds.length && messages ? (
        <Hint>
          <Typography>{t(tKeys.notFound.getKey())}</Typography>
        </Hint>
      ) : (
        <>
          <TransfersList messages={messages as Message[]} />
          <div>{paginationView}</div>
        </>
      )}
    </Loading>
  );
}

export { Messages };
