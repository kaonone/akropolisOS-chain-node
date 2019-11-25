import * as React from 'react';
import { withRouter, RouteComponentProps, Link } from 'react-router-dom';
// import { Grid, IconButton, Typography } from 'components';
import Grid from '@material-ui/core/Grid';
import IconButton from '@material-ui/core/IconButton';
import Typography from '@material-ui/core/Typography';

import { Back } from 'components/icons';
import { withComponent } from 'utils/react';

import { useStyles } from './Header.style';

const LinkIconButton = withComponent(Link)(IconButton);

interface IOwnProps {
  backRoutePath?: string;
  title: React.ReactNode;
  additionalContent?: React.ReactNode;
}

type IProps = IOwnProps & RouteComponentProps;

function HeaderComponent(props: IProps) {
  const { title, backRoutePath, additionalContent } = props;
  const classes = useStyles();

  return (
    <div className={classes.root}>
      <Grid container alignItems="center" spacing={2}>
        {backRoutePath && (
          <Grid item>
            <LinkIconButton to={backRoutePath} className={classes.backButton}>
              <Back />
            </LinkIconButton>
          </Grid>
        )}
        <Grid item xs zeroMinWidth>
          <Typography variant="h5" noWrap className={classes.title}>
            {title}
          </Typography>
        </Grid>
        {!!additionalContent && (
          <Grid item xs={12}>
            {additionalContent}
          </Grid>
        )}
      </Grid>
    </div>
  );
}

export const Header = withRouter(HeaderComponent);
