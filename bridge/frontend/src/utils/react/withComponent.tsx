import * as React from 'react';
import { GetProps } from '_helpers';

interface IBaseProps {
  component?: string | React.ReactType<any>;
}

type StringType = keyof JSX.IntrinsicElements;

type ExtractProps<T extends StringType | React.ComponentType<any>> = T extends StringType
  ? JSX.IntrinsicElements[T]
  : GetProps<Exclude<T, StringType>>;

export function withComponent<C extends StringType | React.ComponentType<any>>(component: C) {
  return function decorate<BProps extends IBaseProps>(BaseComponent: React.ComponentType<BProps>) {
    type CProps = ExtractProps<C>;

    return function WithComponent(props: Omit<BProps, 'component'> & CProps) {
      return <BaseComponent component={component} {...props} />;
    };
  };
}
