import React from 'react';
import { Form } from 'react-final-form';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { TextInputField, DecimalsField } from 'components/form';
import { Typography, Hint, Loading, Button, Grid, CircularProgress } from 'components';
import { useLimitsQuery, LimitKind } from 'generated/bridge-graphql';
import { composeValidators, validateInteger, validatePositiveNumber } from 'utils/validators';
import { DEFAULT_DECIMALS, ETHEREUM_UNIT_NAME } from 'env';

const tKeys = tKeysAll.features.settings.limitsChangingForm;
const tLimitsKeys = tKeysAll.features.settings.limits;

const textFields: LimitKind[] = [
  LimitKind.MaxHostPendingTransactionLimit,
  LimitKind.MaxGuestPendingTransactionLimit,
];

function LimitsChangingForm() {
  const { t } = useTranslate();

  const limitsResult = useLimitsQuery();

  const list = limitsResult.data?.limits;

  const initialFormValues = React.useMemo(
    () =>
      (list &&
        list.reduce(
          (initialValues, limit) => ({
            ...initialValues,
            [limit.kind]: limit.value,
          }),
          {},
        )) ||
      {},
    [list],
  );

  const validate = React.useMemo(() => {
    return composeValidators(validateInteger, validatePositiveNumber);
  }, []);

  const handleCancelButtonClick = React.useCallback(() => {
    // eslint-disable-next-line no-console
    console.log('cancel');
  }, []);

  const onSubmit = React.useCallback(values => {
    // eslint-disable-next-line no-console
    console.log(values);
  }, []);

  return (
    <Loading gqlResults={limitsResult}>
      {!list || !list.length ? (
        <Hint>
          <Typography>{t(tLimitsKeys.notFound.getKey())}</Typography>
        </Hint>
      ) : (
        <Form
          onSubmit={onSubmit}
          initialValues={initialFormValues}
          subscription={{ submitError: true, submitting: true }}
        >
          {({ handleSubmit, submitError, submitting }) => (
            <form onSubmit={handleSubmit}>
              <Grid container spacing={2}>
                {list.map(item => (
                  <Grid item xs={12} key={item.kind}>
                    {textFields.includes(item.kind) ? (
                      <TextInputField
                        validate={validate}
                        name={item.kind}
                        label={t(tLimitsKeys.items[item.kind].getKey())}
                        placeholder={`Enter ${t(
                          tLimitsKeys.items[item.kind].getKey(),
                        ).toLowerCase()}`}
                        variant="outlined"
                        fullWidth
                      />
                    ) : (
                      <DecimalsField
                        validate={validate}
                        baseDecimals={DEFAULT_DECIMALS} // TODO get decimals from the ERC20 Contract
                        baseUnitName={ETHEREUM_UNIT_NAME}
                        name={item.kind}
                        label={t(tLimitsKeys.items[item.kind].getKey())}
                        placeholder={`Enter ${t(
                          tLimitsKeys.items[item.kind].getKey(),
                        ).toLowerCase()}`}
                        InputLabelProps={{
                          shrink: true,
                        }}
                      />
                    )}
                  </Grid>
                ))}
                {!!submitError && (
                  <Grid item xs={12}>
                    <Hint>
                      <Typography color="error">{submitError}</Typography>
                    </Hint>
                  </Grid>
                )}
                <Grid item xs={6}>
                  <Button
                    variant="outlined"
                    color="primary"
                    fullWidth
                    onClick={handleCancelButtonClick}
                  >
                    {t(tKeys.cancelButtonText.getKey())}
                  </Button>
                </Grid>
                <Grid item xs={6}>
                  <Button
                    variant="contained"
                    color="primary"
                    type="submit"
                    fullWidth
                    disabled={submitting}
                  >
                    {submitting ? <CircularProgress size={24} /> : 'submit'}
                  </Button>
                </Grid>
              </Grid>
            </form>
          )}
        </Form>
      )}
    </Loading>
  );
}

export { LimitsChangingForm };
