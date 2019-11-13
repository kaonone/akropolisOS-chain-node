import React, { useCallback } from 'react';
import { Form, FormSpy } from 'react-final-form';
import { FORM_ERROR, FormState } from 'final-form';
import { O } from 'ts-toolbelt';

import { Button, Typography, MenuItem, Box, Balance } from 'components';
import { TextInputField, DecimalsField } from 'components/form';
import { DEFAULT_DECIMALS } from 'env';
import { useApi } from 'services/api';
import { useSubscribable } from 'utils/hooks';
import { getErrorMsg } from 'utils/getErrorMsg';
import { validateRequired, validateEthereumAddress } from 'utils/validators';

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

interface Props {
  onChange?(values: FormData, errors: Errors): void;
}

function validate(values: FormData): Errors {
  return {
    from: validateRequired(values.from.toLowerCase()),
    address: validateRequired(values.address) || validateEthereumAddress(values.address),
    amount: validateRequired(values.amount),
  };
}

function SendingForm({ onChange }: Props) {
  const api = useApi();
  const [accounts, { loaded: accountsLoaded, error: accountsError }] = useSubscribable(
    () => api.getSubstrateAccounts$(),
    [],
  );

  const handleChange = useCallback(
    (formState: FormState<FormData>) => onChange && onChange(formState.values, formState.errors),
    [onChange],
  );

  const onSubmit = useCallback(async ({ from, address, amount }: FormData) => {
    try {
      return await api.sendToEthereum(from, address, amount);
    } catch (error) {
      return { [FORM_ERROR]: getErrorMsg(error) };
    }
  }, []);

  if (!accountsLoaded) {
    return null;
  }

  if (!accounts || !accounts.length || accountsError) {
    return (
      <>
        <Typography color="error">
          You Substrate account can not be found, please install Polkadot.js browser extension and
          create an account.
        </Typography>
        <Typography color="error">
          If you already have account in the extension, please reopen the browser tab.
        </Typography>
      </>
    );
  }

  return (
    <Form<FormData>
      onSubmit={onSubmit}
      subscription={{ submitting: true, submitError: true }}
      initialValues={{ from: accounts[0].address, address: '', amount: '' }}
      validate={validate}
    >
      {({ handleSubmit, submitting, submitError }): React.ReactElement<{}> => (
        <form onSubmit={handleSubmit}>
          <FormSpy<FormData> onChange={handleChange} />
          <TextInputField
            select
            name={fields.from}
            label="From"
            error={false}
            variant="outlined"
            margin="normal"
            fullWidth
          >
            {accounts.map(value => (
              <MenuItem value={value.address} key={value.address}>
                {value.meta.name} ({value.address})
              </MenuItem>
            ))}
          </TextInputField>
          <FormSpy<FormData> subscription={{ errors: true, values: true }}>
            {({ errors, values }: { values: FormData; errors: Errors }) => (
              <TextInputField
                name={fields.address}
                variant="outlined"
                label="To"
                margin="normal"
                InputLabelProps={{
                  shrink: true,
                }}
                helperText={
                  !errors.address &&
                  !!values.address && (
                    <Box color="primary.main">
                      Available: <Balance address={values.address} type="ethereum" />
                    </Box>
                  )
                }
                FormHelperTextProps={{
                  component: 'div',
                }}
                fullWidth
              />
            )}
          </FormSpy>
          <DecimalsField
            baseDecimals={DEFAULT_DECIMALS} // TODO get decimals from the ERC20 Contract
            name={fields.amount}
            label="Amount"
            margin="normal"
            InputLabelProps={{
              shrink: true,
            }}
          />
          {!!submitError && (
            <Typography variant="body1" color="error">
              {submitError}
            </Typography>
          )}
          <Button fullWidth type="submit" variant="contained" color="primary" disabled={submitting}>
            Send{submitting && 'ing'}
          </Button>
        </form>
      )}
    </Form>
  );
}

export { Props as SendingFormProps, SendingForm };
