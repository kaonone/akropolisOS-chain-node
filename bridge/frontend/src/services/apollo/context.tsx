import React from 'react';
import { ApolloProvider as ApolloHooksProvider } from '@apollo/react-hooks';

import { apolloClient } from './apolloClient';

interface Props {
  children: React.ReactNode;
}

export function ApolloProvider({ children }: Props) {
  return <ApolloHooksProvider client={apolloClient}>{children}</ApolloHooksProvider>;
}
