/**
 * Toast Container - Displays toast notifications
 */

import { useEffect, useState } from 'react';
import { makeStyles, shorthands, tokens } from '@fluentui/react-components';
import {
  CheckmarkCircle24Filled,
  DismissCircle24Filled,
  Warning24Filled,
  Info24Filled,
  Dismiss24Regular,
} from '@fluentui/react-icons';
import { getToastManager, type Toast, dismissToast } from '../services/toast';

const useStyles = makeStyles({
  container: {
    position: 'fixed',
    top: '80px',
    right: '20px',
    zIndex: 10000,
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('12px'),
    maxWidth: '400px',
    pointerEvents: 'none',
  },
  toast: {
    pointerEvents: 'auto',
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('12px'),
    ...shorthands.padding('14px', '16px'),
    ...shorthands.borderRadius('8px'),
    backgroundColor: tokens.colorNeutralBackground1,
    boxShadow: tokens.shadow16,
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
    animation: 'slideInRight 0.3s ease-out',
    minWidth: '300px',
  },
  toastSuccess: {
    ...shorthands.borderLeft('4px', 'solid', tokens.colorPaletteGreenBorder2),
  },
  toastError: {
    ...shorthands.borderLeft('4px', 'solid', tokens.colorPaletteRedBorder2),
  },
  toastWarning: {
    ...shorthands.borderLeft('4px', 'solid', tokens.colorPaletteYellowBorder2),
  },
  toastInfo: {
    ...shorthands.borderLeft('4px', 'solid', tokens.colorBrandStroke1),
  },
  icon: {
    fontSize: '20px',
    flexShrink: 0,
  },
  iconSuccess: {
    color: tokens.colorPaletteGreenForeground2,
  },
  iconError: {
    color: tokens.colorPaletteRedForeground2,
  },
  iconWarning: {
    color: tokens.colorPaletteYellowForeground2,
  },
  iconInfo: {
    color: tokens.colorBrandForeground1,
  },
  message: {
    flex: 1,
    fontSize: '14px',
    lineHeight: '1.4',
    color: tokens.colorNeutralForeground1,
  },
  closeButton: {
    cursor: 'pointer',
    color: tokens.colorNeutralForeground3,
    transition: 'color 0.2s',
    flexShrink: 0,
    ':hover': {
      color: tokens.colorNeutralForeground1,
    },
  },
});

export function ToastContainer() {
  const styles = useStyles();
  const [toasts, setToasts] = useState<Toast[]>([]);

  useEffect(() => {
    const manager = getToastManager();
    const unsubscribe = manager.subscribe(setToasts);
    return unsubscribe;
  }, []);

  const getIcon = (type: Toast['type']) => {
    switch (type) {
      case 'success':
        return <CheckmarkCircle24Filled className={`${styles.icon} ${styles.iconSuccess}`} />;
      case 'error':
        return <DismissCircle24Filled className={`${styles.icon} ${styles.iconError}`} />;
      case 'warning':
        return <Warning24Filled className={`${styles.icon} ${styles.iconWarning}`} />;
      case 'info':
        return <Info24Filled className={`${styles.icon} ${styles.iconInfo}`} />;
    }
  };

  const getToastClass = (type: Toast['type']) => {
    switch (type) {
      case 'success':
        return styles.toastSuccess;
      case 'error':
        return styles.toastError;
      case 'warning':
        return styles.toastWarning;
      case 'info':
        return styles.toastInfo;
    }
  };

  if (toasts.length === 0) return null;

  return (
    <>
      <style>
        {`
          @keyframes slideInRight {
            from {
              transform: translateX(100%);
              opacity: 0;
            }
            to {
              transform: translateX(0);
              opacity: 1;
            }
          }
        `}
      </style>
      <div className={styles.container}>
        {toasts.map((toast) => (
          <div key={toast.id} className={`${styles.toast} ${getToastClass(toast.type)}`}>
            {getIcon(toast.type)}
            <div className={styles.message}>{toast.message}</div>
            <Dismiss24Regular
              className={styles.closeButton}
              onClick={() => dismissToast(toast.id)}
            />
          </div>
        ))}
      </div>
    </>
  );
}
