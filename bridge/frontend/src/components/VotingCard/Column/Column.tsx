import * as React from 'react';
import cn from 'classnames';
import { GetProps } from '_helpers';
import Grid from '@material-ui/core/Grid';
import Typography from '@material-ui/core/Typography';

import { useStyles } from './Column.style';

interface IProps {
  xs: GetProps<typeof Grid>['xs'];
  title: string;
  value: React.ReactNode;
  subValue?: string;
  isHighlighted?: boolean;
  icon?: React.ReactElement;
}

function Column(props: IProps) {
  const { xs, icon, title, value, subValue, isHighlighted } = props;
  const classes = useStyles();

  return (
    <Grid item xs={xs} container direction="column">
      <Grid container wrap="nowrap" alignItems="center">
        {icon}
        <Typography
          variant="overline"
          className={cn(classes.title, { [classes.isHighlighted]: isHighlighted })}
        >
          {title}
        </Typography>
      </Grid>
      <Grid container alignItems="baseline">
        <Typography
          variant="h5"
          className={cn(classes.value, { [classes.isHighlighted]: isHighlighted })}
        >
          {value}
        </Typography>
        {subValue && (
          <Typography variant="subtitle1" className={classes.subValue}>
            {subValue}
          </Typography>
        )}
      </Grid>
    </Grid>
  );
}

export { Column };
