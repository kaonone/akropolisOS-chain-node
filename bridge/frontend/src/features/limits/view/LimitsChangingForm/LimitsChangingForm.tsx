import React, { useCallback } from 'react';
import { Form } from 'react-final-form';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { useApi } from 'services/api';
import { TextInputField, DecimalsField } from 'components/form';
import { Typography, Hint, Loading, Button, Grid, CircularProgress } from 'components';
import { useLimitsQuery, LimitKind } from 'generated/bridge-graphql';
import { composeValidators, validateInteger, validatePositiveNumber } from 'utils/validators';
import { useSubscribable } from 'utils/react';
import { DEFAULT_DECIMALS, ETHEREUM_UNIT_NAME } from 'env';

const tKeys = tKeysAll.features.limits.limitsChangingForm;
const tLimitsKeys = tKeysAll.features.limits.limitsList;

const textFields: LimitKind[] = [
  LimitKind.MaxHostPendingTransactionLimit,
  LimitKind.MaxGuestPendingTransactionLimit,
];

interface IProps {
  onCancel: () => void;
}

function LimitsChangingForm(props: IProps) {
  const { onCancel } = props;
  const { t } = useTranslate();
  const api = useApi();
  const [account, accountMeta] = useSubscribable(() => api.getEthAccount$(), []);

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

  const onSubmit = useCallback(
    async values => {
      try {
        await api.createLimitProposal({ fromAddress: account, ...values });
        onCancel();
      } catch (error) {
        throw new Error(error);
      }
    },
    [account],
  );

  return (
    <Loading gqlResults={limitsResult} meta={accountMeta}>
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
                <Grid item xs={12}>
                  <Typography variant="h4" noWrap gutterBottom>
                    {t(tKeys.title.getKey())}
                  </Typography>
                </Grid>
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
                  <Button variant="outlined" color="primary" fullWidth onClick={onCancel}>
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
