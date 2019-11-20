import React from 'react';
import { Form } from 'react-final-form';

import { useApi } from 'services/api';
import { TextInputField } from 'components/form';
import { Grid, Button, Typography, Hint } from 'components';

interface IFormData {
  nodeUrl: string;
}

const fieldNames: { [K in keyof IFormData]: K } = {
  nodeUrl: 'nodeUrl',
};

function Settings() {
  const api = useApi();

  const initialValues = React.useMemo<IFormData>(
    () => ({
      nodeUrl: '',
    }),
    [],
  );

  const onSubmit = React.useCallback((values: IFormData) => {
    const { nodeUrl } = values;
    api.setNodeUrl(nodeUrl);
    document.location.reload();
  }, []);

  const handleResetButtonClick = () => {};

  return (
    <Grid container spacing={4}>
      <Grid item xs={6}>
        <Form
          onSubmit={onSubmit}
          initialValues={initialValues}
          subscription={{ submitting: true, submitError: true }}
        >
          {({ handleSubmit, submitting, submitError }): React.ReactElement<{}> => (
            <form onSubmit={handleSubmit}>
              <Grid container spacing={2}>
                <Grid item xs={12}>
                  <Typography variant="h4" noWrap gutterBottom>
                    Local settings
                  </Typography>
                </Grid>
                <Grid item xs={12}>
                  <TextInputField
                    label="Enter node url"
                    variant="outlined"
                    name={fieldNames.nodeUrl}
                    InputLabelProps={{
                      shrink: true,
                    }}
                    InputProps={{
                      endAdornment: (
                        <Button color="primary" onClick={handleResetButtonClick}>
                          Reset
                        </Button>
                      ),
                    }}
                    fullWidth
                  />
                </Grid>
                {!!submitError && (
                  <Grid item xs={12}>
                    <Hint>
                      <Typography color="error">{submitError}</Typography>
                    </Hint>
                  </Grid>
                )}
                <Grid item xs={12}>
                  <Button
                    fullWidth
                    type="submit"
                    variant="contained"
                    color="primary"
                    disabled={submitting}
                  >
                    Save and reload
                  </Button>
                </Grid>
              </Grid>
            </form>
          )}
        </Form>
      </Grid>
      <Grid item xs={6}>
        <Typography variant="h4" noWrap gutterBottom>
          Bridge settings
        </Typography>
        <Hint>Coming soon</Hint>
      </Grid>
    </Grid>
  );
}

export { Settings };
