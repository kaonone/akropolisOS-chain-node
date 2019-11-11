import * as React from 'react';
import * as R from 'ramda';
import Checkbox from '@material-ui/core/Checkbox';
import { FieldRenderProps } from 'react-final-form';

type Props = FieldRenderProps<string | number | string[] | undefined, HTMLInputElement>;

function CheckboxWrapper({
  input: { checked, name, onChange, ...restInput },
  ...rest
}: Props): React.ReactElement<Props> {
  const restProps = R.omit(['meta'], rest);

  return (
    <Checkbox
      {...restProps}
      name={name}
      inputProps={restInput}
      onChange={onChange}
      checked={checked}
    />
  );
}

export { CheckboxWrapper };
