import { makeStyles, Theme, colors } from 'utils/styles';

export const useStyles = makeStyles((theme: Theme) => ({
  root: {
    borderRadius: '0.25rem',
    backgroundColor: colors.white,
    boxShadow: '0px 1px 0px rgba(0, 0, 0, 0.1)',
  },

  mainInformation: {
    padding: theme.spacing(2),
  },

  address: {
    fontSize: '1.5rem',
    lineHeight: '1.6',
  },

  voting: {
    borderLeft: `solid ${colors.athensGray} 1px`,
    padding: theme.spacing(2),
  },

  toggleExpandIcon: {
    marginRight: theme.spacing(1),
    color: colors.royalPurple,
    cursor: 'pointer',
  },

  expansionPanel: {
    boxShadow: 'none',
  },

  expansionPanelSummary: {
    padding: 0,
  },

  showLimits: {
    color: colors.royalPurple,
  },

  votingIcon: {
    width: '1.25rem',
    marginRight: theme.spacing(0.5),
  },

  votingForIcon: {
    marginRight: theme.spacing(1),
    composes: '$votingIcon',
    color: colors.shamrock,
  },

  votingAgainstIcon: {
    marginRight: theme.spacing(1),
    composes: '$votingIcon',
    color: colors.geraldine,
  },

  votingResult: {
    padding: theme.spacing(2),
    borderLeft: `solid ${colors.athensGray} 1px`,
  },
}));
