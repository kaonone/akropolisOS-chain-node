import * as React from 'react';
import * as R from 'ramda';
import Radio from '@material-ui/core/Radio';
import { FieldRenderProps } from 'react-final-form';

type Props = FieldRenderProps<string, HTMLInputElement>;

function RadioWrapper({
  input: { checked, value, name, onChange, ...restInput },
  ...rest
}: Props): React.ReactElement<Props> {
  const restProps = R.omit(['meta'], rest);

  return (
    <Radio
      {...restProps}
      name={name}
      inputProps={restInput}
      onChange={onChange}
      checked={checked}
      value={value}
    />
  );
}

export { RadioWrapper };
