import * as React from 'react';
import { useCallback } from 'react';
import { Form, Field } from 'react-final-form';
import { FORM_ERROR } from 'final-form';
import { Button, Typography } from '@material-ui/core';

import { TextField } from '~components/form';
import { useApi } from '~components/context';
import { useSubscribable } from '~util/hooks';
import getErrorMsg from '~util/getErrorMsg';

interface FormData {
  address: string;
  amount: string;
}

const fields: { [key in keyof FormData]: key } = {
  address: 'address',
  amount: 'amount',
};

function SendingForm() {
  const api = useApi();
  const [account] = useSubscribable(() => api.getEthAccount$(), []);

  const handleSubmit = useCallback(async ({ address, amount }: FormData) => {
    try {
      if (!account) {
        throw new Error('Source account for token transfer not found');
      }
      await api.sendToSubstrate(account, address, amount);
    } catch (error) {
      return { [FORM_ERROR]: getErrorMsg(error) };
    }
  }, [account]);

  return (
    <Form<FormData> onSubmit={handleSubmit} subscription={{ submitting: true, submitError: true }}>
      {({ handleSubmit, submitting, submitError }): React.ReactElement<{}> => (
        <form onSubmit={handleSubmit}>
          <Field
            name={fields.address}
            component={TextField}
            fullWidth
            variant="outlined"
            label='Address'
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
