/**
 * Keyboard Shortcuts Dialog - Help modal showing all shortcuts
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
import { Keyboard24Regular } from '@fluentui/react-icons';

const useStyles = makeStyles({
  content: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('24px'),
  },
  section: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('12px'),
  },
  sectionTitle: {
    fontSize: '16px',
    fontWeight: tokens.fontWeightSemibold,
    color: tokens.colorBrandForeground1,
    marginBottom: '8px',
  },
  shortcutRow: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    ...shorthands.padding('8px', '12px'),
    backgroundColor: tokens.colorNeutralBackground2,
    ...shorthands.borderRadius('4px'),
  },
  description: {
    fontSize: '14px',
    color: tokens.colorNeutralForeground1,
  },
  keys: {
    display: 'flex',
    ...shorthands.gap('4px'),
  },
  key: {
    ...shorthands.padding('4px', '8px'),
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
    ...shorthands.borderRadius('4px'),
    fontSize: '12px',
    fontFamily: 'monospace',
    fontWeight: 600,
    color: tokens.colorNeutralForeground2,
    boxShadow: '0 1px 2px rgba(0, 0, 0, 0.1)',
  },
  titleContainer: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
  },
});

interface Shortcut {
  description: string;
  keys: string[];
}

interface ShortcutSection {
  title: string;
  shortcuts: Shortcut[];
}

const shortcuts: ShortcutSection[] = [
  {
    title: 'Mode Navigation',
    shortcuts: [
      { description: 'Switch to Compose Mode', keys: ['Ctrl', '1'] },
      { description: 'Switch to Record Mode', keys: ['Ctrl', '2'] },
      { description: 'Switch to Mix Mode', keys: ['Ctrl', '3'] },
      { description: 'Switch to Master Mode', keys: ['Ctrl', '4'] },
      { description: 'Switch to Practice Mode', keys: ['Ctrl', '5'] },
      { description: 'Switch to Distribute Mode', keys: ['Ctrl', '6'] },
    ],
  },
  {
    title: 'General',
    shortcuts: [
      { description: 'Show Keyboard Shortcuts', keys: ['?'] },
      { description: 'Open AI Assistant', keys: ['Ctrl', 'K'] },
      { description: 'Save Project', keys: ['Ctrl', 'S'] },
      { description: 'Export Project', keys: ['Ctrl', 'E'] },
    ],
  },
  {
    title: 'Playback Controls',
    shortcuts: [
      { description: 'Play/Pause', keys: ['Space'] },
      { description: 'Stop', keys: ['Esc'] },
      { description: 'Toggle Metronome', keys: ['M'] },
      { description: 'Toggle Loop', keys: ['L'] },
    ],
  },
  {
    title: 'Recording',
    shortcuts: [
      { description: 'Start/Stop Recording', keys: ['R'] },
      { description: 'Pause Recording', keys: ['P'] },
    ],
  },
];

interface KeyboardShortcutsDialogProps {
  open: boolean;
  onClose: () => void;
}

export function KeyboardShortcutsDialog({ open, onClose }: KeyboardShortcutsDialogProps) {
  const styles = useStyles();

  return (
    <Dialog open={open} onOpenChange={(_, data) => data.open || onClose()}>
      <DialogSurface style={{ maxWidth: '600px' }}>
        <DialogBody>
          <DialogTitle>
            <div className={styles.titleContainer}>
              <Keyboard24Regular />
              <span>Keyboard Shortcuts</span>
            </div>
          </DialogTitle>
          <DialogContent>
            <div className={styles.content}>
              {shortcuts.map((section) => (
                <div key={section.title} className={styles.section}>
                  <div className={styles.sectionTitle}>{section.title}</div>
                  {section.shortcuts.map((shortcut) => (
                    <div key={shortcut.description} className={styles.shortcutRow}>
                      <div className={styles.description}>{shortcut.description}</div>
                      <div className={styles.keys}>
                        {shortcut.keys.map((key, index) => (
                          <span key={index}>
                            <span className={styles.key}>{key}</span>
                            {index < shortcut.keys.length - 1 && <span style={{ margin: '0 4px' }}>+</span>}
                          </span>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              ))}
            </div>
          </DialogContent>
          <DialogActions>
            <Button appearance="primary" onClick={onClose}>
              Got it!
            </Button>
          </DialogActions>
        </DialogBody>
      </DialogSurface>
    </Dialog>
  );
}
