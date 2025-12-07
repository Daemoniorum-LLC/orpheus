/**
 * Message Dialog - Reusable dialog for informational messages
 */

import {
  Dialog,
  DialogSurface,
  DialogBody,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  makeStyles,
  shorthands,
  tokens,
} from '@fluentui/react-components';
import {
  CheckmarkCircle24Regular,
  ErrorCircle24Regular,
  Warning24Regular,
  Info24Regular,
} from '@fluentui/react-icons';

const useStyles = makeStyles({
  content: {
    whiteSpace: 'pre-wrap',
    lineHeight: '1.5',
  },
  titleContainer: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
  },
  successIcon: {
    color: tokens.colorPaletteGreenForeground1,
  },
  errorIcon: {
    color: tokens.colorPaletteRedForeground1,
  },
  warningIcon: {
    color: tokens.colorPaletteYellowForeground1,
  },
  infoIcon: {
    color: tokens.colorBrandForeground1,
  },
});

export type MessageType = 'success' | 'error' | 'warning' | 'info';

interface MessageDialogProps {
  open: boolean;
  onClose: () => void;
  title: string;
  message: string;
  type?: MessageType;
}

export function MessageDialog({ open, onClose, title, message, type = 'info' }: MessageDialogProps) {
  const styles = useStyles();

  const getIcon = () => {
    switch (type) {
      case 'success':
        return <CheckmarkCircle24Regular className={styles.successIcon} />;
      case 'error':
        return <ErrorCircle24Regular className={styles.errorIcon} />;
      case 'warning':
        return <Warning24Regular className={styles.warningIcon} />;
      case 'info':
      default:
        return <Info24Regular className={styles.infoIcon} />;
    }
  };

  return (
    <Dialog open={open} onOpenChange={(_, data) => data.open || onClose()}>
      <DialogSurface>
        <DialogBody>
          <DialogTitle>
            <div className={styles.titleContainer}>
              {getIcon()}
              <span>{title}</span>
            </div>
          </DialogTitle>
          <DialogContent>
            <div className={styles.content}>{message}</div>
          </DialogContent>
          <DialogActions>
            <Button appearance="primary" onClick={onClose}>
              OK
            </Button>
          </DialogActions>
        </DialogBody>
      </DialogSurface>
    </Dialog>
  );
}
