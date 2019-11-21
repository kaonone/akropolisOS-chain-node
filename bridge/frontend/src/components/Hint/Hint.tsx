import React from 'react';

import { useStyles } from './Hint.style';

function Hint(props: React.PropsWithChildren<{}>) {
  const { children } = props;
  const classes = useStyles();

  return <div className={classes.root}>{children}</div>;
}

export { Hint };
