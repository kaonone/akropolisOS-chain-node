import * as React from 'react';

import { useMessagesByIdsQuery } from 'generated/bridge-graphql';
import { Loading } from 'components';

interface IProps {
  ids: string[];
}

function Messages({ ids }: IProps) {
  const { loading, data, error } = useMessagesByIdsQuery({ variables: { ids } });

  return (
    <Loading meta={{ loaded: !loading, error: error && error.message }}>
      <pre>{data && JSON.stringify(data.messages, null, 2)}</pre>
    </Loading>
  );
}

export { Messages };
