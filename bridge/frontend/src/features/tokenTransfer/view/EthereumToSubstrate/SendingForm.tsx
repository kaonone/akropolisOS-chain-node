import React, { useCallback } from 'react';
import { Form, FormSpy } from 'react-final-form';
import { FORM_ERROR } from 'final-form';
import { O } from 'ts-toolbelt';

import { Button, Typography, Grid, Box, Balance } from 'components';
import { TextInputField, DecimalsField } from 'components/form';
import { ITranslateKey } from 'services/i18n';
import { useApi } from 'services/api';
import { useSubscribable } from 'utils/hooks';
import { getErrorMsg } from 'utils/getErrorMsg';
import { validateRequired, validateSubstrateAddress } from 'utils/validators';
import { DEFAULT_DECIMALS, ETHEREUM_UNIT_NAME } from 'env';

interface FormData {
  address: string;
  amount: string;
}

const fields: { [key in keyof FormData]: key } = {
  address: 'address',
  amount: 'amount',
};

type Errors = Partial<O.Update<FormData, keyof FormData, ITranslateKey>>;

function validate(values: FormData): Errors {
  return {
    address: validateRequired(values.address) || validateSubstrateAddress(values.address),
    amount: validateRequired(values.amount),
  };
}

function SendingForm() {
  const api = useApi();
  const [account] = useSubscribable(() => api.getEthAccount$(), []);

  const onSubmit = useCallback(
    async ({ address, amount }: FormData) => {
      try {
        if (!account) {
          throw new Error('Source account for token transfer not found');
        }
        return await api.sendToSubstrate(account, address, amount);
      } catch (error) {
        return { [FORM_ERROR]: getErrorMsg(error) };
      }
    },
    [account],
  );

  return (
    <Form<FormData>
      onSubmit={onSubmit}
      subscription={{ submitting: true, submitError: true }}
      initialValues={{ address: '', amount: '' }}
      validate={validate}
    >
      {({ handleSubmit, submitting, submitError }): React.ReactElement<{}> => (
        <form onSubmit={handleSubmit}>
          <Grid container spacing={2}>
            <Grid item xs>
              <FormSpy<FormData> subscription={{ errors: true, values: true }}>
                {({ errors, values }: { values: FormData; errors: Errors }) => (
                  <TextInputField
                    name={fields.address}
                    fullWidth
                    variant="outlined"
                    label="Address"
                    margin="normal"
                    InputLabelProps={{
                      shrink: true,
                    }}
                    helperText={
                      !errors.address &&
                      !!values.address && (
                        <Box color="primary.main">
                          Available: <Balance address={values.address} type="substrate" />
                        </Box>
                      )
                    }
                    FormHelperTextProps={{
                      component: 'div',
                    }}
                  />
                )}
              </FormSpy>
            </Grid>
          </Grid>
          <DecimalsField
            baseDecimals={DEFAULT_DECIMALS} // TODO get decimals from the ERC20 Contract
            baseUnitName={ETHEREUM_UNIT_NAME}
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

export { SendingForm };
