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
