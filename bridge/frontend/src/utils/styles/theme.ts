import { createMuiTheme, Theme } from '@material-ui/core/styles';

export { Theme };

export const theme: Theme = createMuiTheme({
  palette: {
    primary: {
      main: '#6931b6',
    },
  },
  overrides: {
    MuiFormHelperText: {
      root: {
        '&:empty': {
          display: 'none',
        },
      },
    },
  },
});

export const gradients = {
  purple: 'linear-gradient(360deg, #7357D2 0%, #8E41DC 100%)',
};
