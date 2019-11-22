import React from 'react';

import { useApi } from 'services/api';
import { useSubscribable } from 'utils/hooks';
import { Message, useCurrentMessagesByIdsSubscription } from 'generated/bridge-graphql';
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
  const [transfers, transfersMeta] = useSubscribable(() => api.getTransfers$(), [], []);

  const ids = transfers.map(transaction => transaction.id);
  const { items: paginatedIds, paginationView } = usePagination(ids);

  const { loading, data, error } = useCurrentMessagesByIdsSubscription({
    variables: { ids: (paginatedIds.length && paginatedIds) || mockIds },
  });

  const messages = React.useMemo(
    () =>
      paginatedIds
        .map(
          id =>
            data?.messages?.find(item => item.id === id) || transfers.find(item => item.id === id),
        )
        .filter((item): item is Message => !!item),
    [paginatedIds, data?.messages, transfers],
  );

  return (
    <Loading meta={[transfersMeta, { loaded: !loading, error: error && error.message }]}>
      {!messages.length ? (
        <Hint>
          <Typography>{t(tKeys.notFound.getKey())}</Typography>
        </Hint>
      ) : (
        <>
          <TransfersList messages={messages} />
          <div>{paginationView}</div>
        </>
      )}
    </Loading>
  );
}

export { Messages };
