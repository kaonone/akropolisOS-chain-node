import React, { useState } from 'react';
import { ApolloProvider as ApolloHooksProvider } from '@apollo/react-hooks';
import {
  introspectSchema,
  makeRemoteExecutableSchema,
  addMockFunctionsToSchema,
  MockList,
} from 'graphql-tools';
import { SchemaLink } from 'apollo-link-schema';

import { getEnv } from 'core/getEnv';

import { defaultApolloClient, apolloLink, createApolloClient } from './apolloClient';

interface Props {
  children: React.ReactNode;
}

export function ApolloProvider({ children }: Props) {
  const { isMockServer } = getEnv();
  const [apolloClient, setApolloClient] = useState(null as any);

  const createMockApolloClient = async () => {
    const schema = await introspectSchema(apolloLink);
    const executableSchema = makeRemoteExecutableSchema({ schema });

    const mocks = {
      Query: () => ({
        limits: () => new MockList([10, 10]),
        limitProposal: () => ({
          ethAddress: () => '0x0000000000000000000000000000000000000000000000000000000000000000',
        }),
      }),
      BigInt: () => '123456',
    };

    addMockFunctionsToSchema({
      schema: executableSchema,
      mocks,
    });

    const mockLink = new SchemaLink({ schema: executableSchema });
    setApolloClient(createApolloClient(mockLink));
  };

  if (isMockServer && apolloClient) {
    return <ApolloHooksProvider client={apolloClient}>{children}</ApolloHooksProvider>;
  }

  if (isMockServer) {
    createMockApolloClient();
    return <>Mock server is loading...</>;
  }

  return <ApolloHooksProvider client={defaultApolloClient}>{children}</ApolloHooksProvider>;
}
