import * as React from 'react';
import { FieldRenderProps } from 'react-final-form';

import { getFieldWithComponent, useOnChangeState } from 'utils/react';

interface IOwnProps<T> {
  fieldValue: T; // final-form intercepts the 'value' property
  compare?: (prev: T, current: T) => boolean;
}

type IProps<T> = Omit<FieldRenderProps<any, HTMLElement>, 'value'> & IOwnProps<T>;

function SpyFieldComponent<T>(props: IProps<T>) {
  const { input, fieldValue, compare } = props;
  const { onChange } = input;

  useOnChangeState(fieldValue, compare || defaultCompare, (_prev, current) => onChange(current));

  return <input {...input} type="hidden" />;
}

function defaultCompare<T>(prev: T, current: T) {
  return prev === current;
}

export const SpyField = getFieldWithComponent(SpyFieldComponent);
