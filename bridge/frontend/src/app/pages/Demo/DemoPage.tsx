import * as React from 'react';
import Container from '@material-ui/core/Container';

import { Typography } from 'components';

import { Messages } from './Messages/Messages';

const ids = [
  '0x01f0f84df157d3324bed08dd9c2408402ec3ceaa0778465f0015d67b99d06812',
  '0x021b75496294ad708a923e6b9554d3c5382b2127dffdd35f194d4f0ae42ed3c4',
];

export function DemoPage() {
  return (
    <Container fixed>
      <Typography>Demo page</Typography>
      <Messages ids={ids} />
    </Container>
  );
}
