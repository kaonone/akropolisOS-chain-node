import React from 'react';
import LinearProgress, { LinearProgressProps } from '@material-ui/core/LinearProgress';
import CircularProgress, { CircularProgressProps } from '@material-ui/core/CircularProgress';
import Typography from '@material-ui/core/Typography';
import { makeStyles } from '@material-ui/core/styles';

import { CommunicationState } from 'utils/react';

interface IMeta {
  loaded: boolean;
  error?: string | null;
}

type MaybeArray<T> = T | T[];
type ProgressType = 'linear' | 'circle';

interface IProps<V extends ProgressType> {
  children?: React.ReactNode;
  meta?: MaybeArray<IMeta>;
  communication?: MaybeArray<CommunicationState<any, any>>;
  component?: React.ComponentType;
  progressVariant?: V;
  progressProps?: {
    linear: LinearProgressProps;
    circle: CircularProgressProps;
  }[V];
  ignoreError?: boolean;
}

const useStyles = makeStyles({
  linearProgress: {
    flexGrow: 1,
  },
});

export function Loading<T extends ProgressType>(props: IProps<T>) {
  const classes = useStyles();
  const {
    children,
    progressVariant,
    progressProps,
    component,
    ignoreError,
    meta = [],
    communication = [],
  } = props;
  const metas = Array.isArray(meta) ? meta : [meta];
  const communications = Array.isArray(communication) ? communication : [communication];

  const loadedMetas = metas.every(value => value.loaded);
  const { error: metasError } = metas.find(value => value.error) || { error: null };

  const loadedCommunications = communications.every(value => value.status !== 'pending');
  const { error: communicationsError } = communications.find(value => value.error) || {
    error: null,
  };

  const loaded = loadedMetas && loadedCommunications;
  const error = metasError || communicationsError;

  const Wrapper = component || React.Fragment;

  const needToShowError = !!error && !ignoreError;

  return (
    <>
      {!loaded && (
        <Wrapper>
          {progressVariant === 'circle' ? (
            <CircularProgress {...(progressProps as CircularProgressProps)} />
          ) : (
            <LinearProgress
              className={classes.linearProgress}
              {...(progressProps as LinearProgressProps)}
            />
          )}
        </Wrapper>
      )}
      {loaded && needToShowError && (
        <Wrapper>
          <Typography color="error">{error}</Typography>
        </Wrapper>
      )}
      {loaded && !needToShowError && children}
    </>
  );
}
