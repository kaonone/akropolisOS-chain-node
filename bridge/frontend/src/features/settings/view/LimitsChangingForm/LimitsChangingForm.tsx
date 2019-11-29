import React from 'react';
import { Form } from 'react-final-form';

import { useTranslate, tKeys as tKeysAll } from 'services/i18n';
import { MakeTableType } from 'components/Table/Table';
import { TextInputField } from 'components/form';
import {
  Typography,
  Hint,
  Table as GeneralTable,
  Loading,
  Button,
  Grid,
  CircularProgress,
} from 'components';
import { useLimitsQuery, Limit } from 'generated/bridge-graphql';
import { composeValidators, validateInteger, validatePositiveNumber } from 'utils/validators';

const Table = GeneralTable as MakeTableType<Limit>;

const tKeys = tKeysAll.features.settings.limitsChangingForm;
const tLimitsKeys = tKeysAll.features.settings.limits;

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
    <>
      <Typography variant="h4" noWrap gutterBottom>
        {t(tKeys.title.getKey())}
      </Typography>
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
                <Grid container spacing={3}>
                  <Grid item xs={12}>
                    <Table data={list}>
                      <Table.Column>
                        <Table.Head>{t(tLimitsKeys.kind.getKey())}</Table.Head>
                        <Table.Cell>
                          {({ data }) => t(tLimitsKeys.items[data.kind].getKey())}
                        </Table.Cell>
                      </Table.Column>
                      <Table.Column>
                        <Table.Head>{t(tLimitsKeys.value.getKey())}</Table.Head>
                        <Table.Cell>
                          {({ data }) => (
                            <TextInputField
                              validate={validate}
                              name={data.kind}
                              placeholder={`Enter ${t(
                                tLimitsKeys.items[data.kind].getKey(),
                              ).toLowerCase()}`}
                              variant="outlined"
                              fullWidth
                            />
                          )}
                        </Table.Cell>
                      </Table.Column>
                    </Table>
                  </Grid>
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
    </>
  );
}

export { LimitsChangingForm };
