import * as React from 'react';
import cn from 'classnames';
import Grid from '@material-ui/core/Grid';
import Typography from '@material-ui/core/Typography';

import { useStyles } from './VotingProgressBar.style';

interface IVotingProgressBarProps {
  title: string;
  value: number;
  type: 'for' | 'against';
}

function VotingProgressBar(props: IVotingProgressBarProps) {
  const { title, value, type } = props;
  const classes = useStyles();

  return (
    <div className={classes.root}>
      <Grid container justify="space-between" wrap="nowrap">
        <Typography component="span" variant="subtitle1">
          {title}
        </Typography>
        <Typography component="span" variant="subtitle1">
          {value}
        </Typography>
      </Grid>
      <div className={classes.progressBar}>
        <div
          className={cn(classes.progressBarValue, {
            [classes.green]: type === 'for',
            [classes.red]: type === 'against',
          })}
          style={{ width: `${value.toFixed(2)}%` }}
        />
      </div>
    </div>
  );
}

export { VotingProgressBar };
