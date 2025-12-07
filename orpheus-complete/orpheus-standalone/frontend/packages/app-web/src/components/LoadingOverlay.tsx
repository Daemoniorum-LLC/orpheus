/**
 * Loading Overlay - Global loading indicator with progress tracking
 */

import { makeStyles, shorthands, tokens, Spinner, Button, ProgressBar } from '@fluentui/react-components';
import { Dismiss24Regular } from '@fluentui/react-icons';
import { useAppStore } from '../store/app-store';

const useStyles = makeStyles({
  overlay: {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 9999,
    flexDirection: 'column',
    ...shorthands.gap('16px'),
  },
  content: {
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.padding('32px'),
    ...shorthands.borderRadius('8px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
    boxShadow: tokens.shadow64,
    textAlign: 'center',
    minWidth: '400px',
    maxWidth: '500px',
  },
  message: {
    marginTop: '16px',
    fontSize: '14px',
    color: tokens.colorNeutralForeground2,
    marginBottom: '8px',
  },
  progressContainer: {
    marginTop: '20px',
    width: '100%',
  },
  progressText: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground3,
    marginTop: '8px',
    fontFamily: 'monospace',
  },
  actions: {
    marginTop: '20px',
    display: 'flex',
    justifyContent: 'center',
  },
});

export function LoadingOverlay() {
  const styles = useStyles();
  const {
    isLoading,
    loadingMessage,
    loadingProgress,
    loadingCancellable,
    cancelLoading,
  } = useAppStore();

  if (!isLoading) {
    return null;
  }

  const hasProgress = loadingProgress >= 0;
  const progressValue = hasProgress ? loadingProgress / 100 : undefined;

  const handleCancel = () => {
    if (cancelLoading) {
      cancelLoading();
    }
  };

  return (
    <div className={styles.overlay}>
      <div className={styles.content}>
        <Spinner size="large" />
        {loadingMessage && <div className={styles.message}>{loadingMessage}</div>}

        {hasProgress && (
          <div className={styles.progressContainer}>
            <ProgressBar
              value={progressValue}
              thickness="large"
              color="brand"
            />
            <div className={styles.progressText}>
              {Math.round(loadingProgress)}% complete
            </div>
          </div>
        )}

        {loadingCancellable && cancelLoading && (
          <div className={styles.actions}>
            <Button
              appearance="secondary"
              icon={<Dismiss24Regular />}
              onClick={handleCancel}
            >
              Cancel
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}
