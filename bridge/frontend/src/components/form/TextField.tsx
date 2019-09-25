import * as React from 'react';
import { FieldRenderProps } from 'react-final-form';
import TextField from '@material-ui/core/TextField';

type Props = FieldRenderProps<string, HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>

function TextFieldWrapper ({ input: { name, onChange, value, ...restInput }, meta, ...rest }: Props): React.ReactElement<Props> {
  const showError = ((meta.submitError && !meta.dirtySinceLastSubmit) || meta.error) && meta.touched;

  return (
    <TextField
      {...rest}
      name={name}
      helperText={showError ? meta.error || meta.submitError : undefined}
      error={showError}
      inputProps={restInput}
      onChange={onChange}
      value={value}
    />
  );
}

export default TextFieldWrapper;
