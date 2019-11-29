import { makeStyles } from 'utils/styles';

export const useStyles = makeStyles(() => {
  return {
    tooltip: {
      cursor: 'pointer',
      borderBottom: '1px dashed',
    },
  } as const;
});
