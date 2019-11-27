import React from 'react';
import LinearProgress, { LinearProgressProps } from '@material-ui/core/LinearProgress';
import CircularProgress, { CircularProgressProps } from '@material-ui/core/CircularProgress';
import Typography from '@material-ui/core/Typography';
import { makeStyles } from '@material-ui/core/styles';
import { QueryResult } from '@apollo/react-common';

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
  gqlResults?: MaybeArray<QueryResult>;
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

function toArray<T>(value: MaybeArray<T>): T[] {
  return Array.isArray(value) ? value : [value];
}

function communicationsToMetas(values: MaybeArray<CommunicationState<any, any>>): IMeta[] {
  return toArray(values).map<IMeta>(value => ({
    loaded: value.status !== 'pending',
    error: value.error,
  }));
}

function gqlResultsToMetas(values: MaybeArray<QueryResult>): IMeta[] {
  return toArray(values).map<IMeta>(value => ({
    loaded: !!value.data || !value.loading,
    error: value.error?.message,
  }));
}

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
    gqlResults = [],
  } = props;
  const metas = [
    ...toArray(meta),
    ...communicationsToMetas(communication),
    ...gqlResultsToMetas(gqlResults),
  ];

  const loaded = metas.every(value => value.loaded);
  const { error } = metas.find(value => value.error) || { error: null };

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
