import { makeStyles } from 'utils/styles';

export const useStyles = makeStyles(() => {
  return {
    tooltip: {
      cursor: 'pointer',
    },
  } as const;
});
