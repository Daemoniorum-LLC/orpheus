/**
 * Confirm Dialog - Reusable confirmation dialog for destructive actions
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
import { Warning24Regular, Delete24Regular, Info24Regular } from '@fluentui/react-icons';

const useStyles = makeStyles({
  content: {
    lineHeight: '1.5',
  },
  titleContainer: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
  },
  warningIcon: {
    color: tokens.colorPaletteYellowForeground1,
  },
  dangerIcon: {
    color: tokens.colorPaletteRedForeground1,
  },
  infoIcon: {
    color: tokens.colorBrandForeground1,
  },
});

export type ConfirmDialogType = 'warning' | 'danger' | 'info';

interface ConfirmDialogProps {
  open: boolean;
  onConfirm: () => void;
  onCancel: () => void;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  type?: ConfirmDialogType;
}

export function ConfirmDialog({
  open,
  onConfirm,
  onCancel,
  title,
  message,
  confirmText = 'Confirm',
  cancelText = 'Cancel',
  type = 'warning',
}: ConfirmDialogProps) {
  const styles = useStyles();

  const getIcon = () => {
    switch (type) {
      case 'danger':
        return <Delete24Regular className={styles.dangerIcon} />;
      case 'warning':
        return <Warning24Regular className={styles.warningIcon} />;
      case 'info':
      default:
        return <Info24Regular className={styles.infoIcon} />;
    }
  };

  const handleConfirm = () => {
    onConfirm();
    onCancel(); // Close dialog after confirming
  };

  return (
    <Dialog open={open} onOpenChange={(_, data) => data.open || onCancel()}>
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
            <Button appearance="secondary" onClick={onCancel}>
              {cancelText}
            </Button>
            <Button
              appearance="primary"
              onClick={handleConfirm}
              style={
                type === 'danger'
                  ? { backgroundColor: tokens.colorPaletteRedBackground3 }
                  : undefined
              }
            >
              {confirmText}
            </Button>
          </DialogActions>
        </DialogBody>
      </DialogSurface>
    </Dialog>
  );
}
