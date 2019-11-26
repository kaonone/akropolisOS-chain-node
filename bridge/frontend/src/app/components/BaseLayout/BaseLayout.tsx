import * as React from 'react';
import { GetProps } from '_helpers';

import { RowsLayout } from 'components';

import { Header } from '../Header/Header';
import { PageNavigation } from '../PageNavigation/PageNavigation';
import { useStyles } from './BaseLayout.style';

interface IProps {
  title: React.ReactNode;
  actions?: React.ReactNode[];
  backRoutePath?: string;
  additionalHeaderContent?: React.ReactNode;
  showBalances?: boolean;
  showEra?: boolean;
  hidePageNavigation?: boolean;
  children: React.ReactNode;
}

function BaseLayout(props: IProps) {
  const { children, backRoutePath, title, additionalHeaderContent, hidePageNavigation } = props;

  const headerProps: GetProps<typeof Header> = {
    backRoutePath,
    title,
    additionalContent: additionalHeaderContent,
  };

  const classes = useStyles();

  return (
    <RowsLayout spacing={4} className={classes.rootRowsLayout}>
      <RowsLayout.ContentBlock>
        <Header {...headerProps} />
        {!hidePageNavigation && <PageNavigation />}
      </RowsLayout.ContentBlock>
      <RowsLayout.ContentBlock fillIn>{children}</RowsLayout.ContentBlock>
    </RowsLayout>
  );
}

export { BaseLayout };
