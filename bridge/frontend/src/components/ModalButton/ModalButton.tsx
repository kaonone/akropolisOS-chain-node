import * as React from 'react';
import { GetProps } from '_helpers';
import Button from '@material-ui/core/Button';
import Dialog, { DialogProps } from '@material-ui/core/Dialog';
import DialogContent from '@material-ui/core/DialogContent';

import { useStyles } from './ModalButton.style';

interface IChildrenProps {
  closeModal(): void;
}

type ButtonProps = Pick<GetProps<typeof Button>, 'variant' | 'color' | 'disabled' | 'fullWidth'>;

interface IProps extends ButtonProps {
  dialogMaxWidth?: DialogProps['maxWidth'];
  content: React.ReactNode;
  children: (props: IChildrenProps) => React.ReactNode;
}

function ModalButton(props: IProps) {
  const classes = useStyles();
  const { children, content, dialogMaxWidth, ...rest } = props;
  const [isOpened, setIsOpened] = React.useState(false);

  const openModal = React.useCallback(() => setIsOpened(true), []);
  const closeModal = React.useCallback(() => setIsOpened(false), []);

  return (
    <>
      <Button {...rest} onClick={openModal}>
        {content}
      </Button>
      <Dialog fullWidth maxWidth={dialogMaxWidth || 'sm'} open={isOpened} onClose={closeModal}>
        <DialogContent className={classes.dialogContent}>
          {typeof children === 'function' ? children({ closeModal }) : children}
        </DialogContent>
      </Dialog>
    </>
  );
}

export { ModalButton };
