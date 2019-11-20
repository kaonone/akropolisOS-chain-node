import { Theme, colors, makeStyles } from 'utils/styles';

export const useStyles = makeStyles((theme: Theme) => {
  return {
    root: {
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      minHeight: theme.spacing(6),
      padding: theme.spacing(1.5),
      borderRadius: '0.25rem',
      backgroundColor: colors.whiteLilac,
      textAlign: 'center',
    },
  };
});
