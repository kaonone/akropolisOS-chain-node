import * as React from 'react';
import Select from '@material-ui/core/Select';
import FormControl, { FormControlProps } from '@material-ui/core/FormControl';
import InputLabel from '@material-ui/core/InputLabel';
import FormHelperText from '@material-ui/core/FormHelperText';

import { FieldRenderProps } from 'react-final-form';

interface Props extends FieldRenderProps<string, HTMLElement> {
  label: string;
  formControlProps: FormControlProps;
}

function FormHelperTextWrapper ({
  input: { name, value, onChange, ...restInput },
  meta,
  label,
  formControlProps,
  ...rest
}: Props): React.ReactElement<Props> {
  const showError = ((meta.submitError && !meta.dirtySinceLastSubmit) || meta.error) && meta.touched;

  return (
    <FormControl {...formControlProps} error={showError}>
      <InputLabel htmlFor={name}>{label}</InputLabel>

      <Select
        {...rest}
        name={name}
        onChange={onChange}
        inputProps={restInput}
        value={value}
      />

      {showError &&
        <FormHelperText>{meta.error || meta.submitError}</FormHelperText>
      }
    </FormControl>
  );
}

export default FormHelperTextWrapper;
