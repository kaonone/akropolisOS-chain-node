import * as React from 'react';
import Typography from '@material-ui/core/Typography';
import LinearProgress from '@material-ui/core/LinearProgress';
import Button from '@material-ui/core/Button';
import Grid from '@material-ui/core/Grid';

import { ETH_NETWORK_CONFIG } from 'env';
import { useSubscribable } from 'utils/hooks';
import { useApi } from 'services/api';

function EthereumValidators() {
  const api = useApi();

  const [validators, { error, loaded }] = useSubscribable(() => api.getEthValidators$(), [], []);

  return (
    <div>
      {!loaded && !error && <LinearProgress />}
      {!!error && <Typography color="error">{error}</Typography>}
      {loaded &&
        !error &&
        (!validators.length ? (
          <Typography color="error">Validators not found</Typography>
        ) : (
          <Grid container spacing={1}>
            {validators.map((validator, index) => (
              <Grid item key={validator}>
                <Button
                  target="_blank"
                  rel="noopener noreferrer"
                  variant="outlined"
                  href={`${ETH_NETWORK_CONFIG.etherskanDomain}address/${validator}`}
                >
                  # {index}
                </Button>
              </Grid>
            ))}
          </Grid>
        ))}
    </div>
  );
}

export { EthereumValidators };
