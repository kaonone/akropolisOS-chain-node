import * as React from 'react';
import { useCallback } from 'react';
import { Form, Field } from 'react-final-form';
import { FORM_ERROR } from 'final-form';
import { Button, Typography, MenuItem } from '@material-ui/core';
import { O } from 'ts-toolbelt';

import { TextField, Select } from '~components/form';
import { useApi } from '~components/context';
import { useSubscribable } from '~util/hooks';
import getErrorMsg from '~util/getErrorMsg';
import { validateRequired, validateEthereumAddress, validateFloat } from '~util/validators';
import { DEFAULT_DECIMALS } from '~env';

interface FormData {
  address: string;
  amount: string;
  from: string;
}

const fields: { [key in keyof FormData]: key } = {
  address: 'address',
  amount: 'amount',
  from: 'from',
};

type Errors = Partial<O.Update<FormData, keyof FormData, string>>;

function validate(values: FormData): Errors {
  return {
    from: validateRequired(values.from.toLowerCase()),
    address: validateRequired(values.address) || validateEthereumAddress(values.address),
    amount: validateRequired(values.amount) || validateFloat(values.amount, DEFAULT_DECIMALS),
  };
}

function SendingForm() {
  const api = useApi();
  const [accounts, { loaded: accountsLoaded }] = useSubscribable(() => api.getSubstrateAccounts$(), []);

  const handleSubmit = useCallback(async ({ from, address, amount }: FormData) => {
    try {
      await api.sendToEthereum(from, address, amount);
    } catch (error) {
      return { [FORM_ERROR]: getErrorMsg(error) };
    }
  }, []);

  if (!accountsLoaded) {
    return null;
  }

  if (!accounts || !accounts.length) {
    return (
      <Typography color="error">
        You don't have any Substrate accounts, you need to create an account in the browser extension Polkadot.js
      </Typography>
    )
  }

  return (
    <Form<FormData>
      onSubmit={handleSubmit}
      subscription={{ submitting: true, submitError: true }}
      initialValues={{ from: accounts[0].address, address: '', amount: '' }}
      validate={validate}
    >
      {({ handleSubmit, submitting, submitError }): React.ReactElement<{}> => (
        <form onSubmit={handleSubmit}>
          <Field
            name={fields.from}
            component={Select as any}
            label='From'
            error={false}
            formControlProps={{
              fullWidth: true,
              variant: "outlined",
              margin: "normal"
            }}
          >
            {accounts.map(value => (
              <MenuItem value={value.address} key={value.address}>{value.meta.name}</MenuItem>
            ))}
          </Field>
          <Field
            name={fields.address}
            component={TextField}
            fullWidth
            variant="outlined"
            label='To'
            margin="normal"
            error={false}
            InputProps={{
              autoFocus: true
            }}
            InputLabelProps={{
              shrink: true
            }}
          />
          <Field
            name={fields.amount}
            component={TextField}
            fullWidth
            variant="outlined"
            label='Amount'
            margin="normal"
            error={false}
            InputLabelProps={{
              shrink: true
            }}
          />
          {!!submitError && <Typography variant='body1' color="error">{submitError}</Typography>}
          <Button fullWidth type="submit" variant="contained" color="primary" disabled={submitting}>
            Send{submitting && 'ing'}
          </Button>
        </form>
      )}
    </Form>
  );
}

export default SendingForm;
