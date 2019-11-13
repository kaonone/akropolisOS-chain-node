import React, { useState, useEffect, useMemo } from 'react';
import { Subscribable } from 'rxjs';

import { getErrorMsg } from '../getErrorMsg';

interface IMeta {
  loaded: boolean;
  error: string | null;
  updatedAt: number;
}

type Result<T> = [T, IMeta];

function useSubscribable<T>(getTarget: () => Subscribable<T>, deps: any[]): Result<T | undefined>;
function useSubscribable<T>(getTarget: () => Subscribable<T>, deps: any[], fallback: T): Result<T>;
function useSubscribable<T>(
  getTarget: () => Subscribable<T>,
  deps: any[],
  fallback?: T,
): Result<T | undefined> {
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [updatedAt, setUpdatedAt] = useState(() => Date.now());
  const [value, setValue] = useState<T | undefined>(fallback);

  const resetState = React.useCallback(() => {
    setLoaded(false);
    setError(null);
    setUpdatedAt(Date.now());
    setValue(fallback);
  }, [fallback]);

  const target = useMemo(getTarget, deps);

  useEffect(() => {
    resetState();

    const subscription = target.subscribe({
      next: nextValue => {
        setLoaded(true);
        setError(null);
        setUpdatedAt(Date.now());
        setValue(nextValue);
      },
      error: err => {
        setLoaded(true);
        setError(getErrorMsg(err));
      },
    });

    return () => subscription.unsubscribe();
  }, [target]);

  const meta: IMeta = useMemo(
    () => ({
      loaded,
      updatedAt,
      error,
    }),
    [loaded, updatedAt, error],
  );

  return [value, meta];
}

export { useSubscribable };
