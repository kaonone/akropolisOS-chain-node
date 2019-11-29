import { Theme, makeStyles, colors } from 'utils/styles';

export const useStyles = makeStyles((theme: Theme) => ({
  root: {},

  title: {
    color: colors.topaz,

    '&$isHighlighted': {
      color: colors.royalPurple,
    },
  },

  value: {
    color: colors.haiti,

    '&$isHighlighted': {
      color: colors.royalPurple,
    },
  },

  subValue: {
    marginLeft: theme.spacing(0.5),
    color: colors.frenchGray,
  },

  isHighlighted: {},
}));
