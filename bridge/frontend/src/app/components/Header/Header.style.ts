import { makeStyles, Theme, gradients } from 'utils/styles';

export const useStyles = makeStyles((theme: Theme) => ({
  root: {
    padding: `${theme.spacing(3.5)}px ${theme.spacing(3)}px`,
    background: gradients.purple,
    borderRadius: 4,
  },

  backButton: {
    color: '#fff',
  },

  title: {
    color: '#fff',
  },
}));
