import React, { useState } from 'react';
import cn from 'classnames';
import Typography from '@material-ui/core/Typography';
import Tooltip from '@material-ui/core/Tooltip';
import CopyToClipboard from 'react-copy-to-clipboard';

import { useStyles } from './ShortAddress.style';

function ShortAddress({ address, className }: { address: string; className?: string }) {
  const classes = useStyles();

  const [tooltipTitle, setTooltipTitle] = useState('copy');

  const shortAddress = `${address.substr(0, 4)}...${address.substr(-4, 4)}`;

  const handleCopy = () => {
    setTooltipTitle('copied!');
  };

  const handleTooltipClose = () => {
    setTooltipTitle('copy');
  };

  return (
    <Tooltip
      className={cn(classes.tooltip, className)}
      title={tooltipTitle}
      onClose={handleTooltipClose}
      placement="bottom"
    >
      <CopyToClipboard text={address} onCopy={handleCopy}>
        <Typography>{shortAddress}</Typography>
      </CopyToClipboard>
    </Tooltip>
  );
}

export { ShortAddress };
