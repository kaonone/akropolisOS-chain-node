import { makeStyles, colors } from 'utils/styles';

export const useStyles = makeStyles(() => ({
  root: {},

  green: {},
  red: {},

  progressBar: {
    borderRadius: '0.25rem',
    height: '2.1px', // if specify exactly 2, some time sizes between progresses may be different
    overflow: 'hidden',
    backgroundColor: colors.athensGray,
  },

  progressBarValue: {
    height: '100%',

    '&$red': {
      backgroundColor: colors.geraldine,
    },

    '&$green': {
      backgroundColor: colors.shamrock,
    },
  },
}));
