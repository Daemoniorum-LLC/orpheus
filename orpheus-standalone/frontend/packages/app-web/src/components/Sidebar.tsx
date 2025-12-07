/**
 * Sidebar - Track list and project navigation
 */

import {
  makeStyles,
  shorthands,
  tokens,
  Button,
} from '@fluentui/react-components';
import { ChevronLeft24Regular, ChevronRight24Regular } from '@fluentui/react-icons';
import { useAppStore, useProject } from '../store/app-store';

const useStyles = makeStyles({
  wrapper: {
    display: 'flex',
    position: 'relative',
    flexShrink: 0,
  },
  sidebar: {
    width: '280px',
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.borderRight('1px', 'solid', tokens.colorNeutralStroke1),
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.transition('width', '200ms', 'ease'),
  },
  sidebarCollapsed: {
    width: '0',
    ...shorthands.borderRight('none'),
    overflow: 'hidden',
  },
  toggleButton: {
    position: 'absolute',
    right: '-12px',
    top: '50%',
    transform: 'translateY(-50%)',
    zIndex: 10,
    minWidth: '24px',
    width: '24px',
    height: '48px',
    ...shorthands.padding('0'),
    ...shorthands.borderRadius('0', '4px', '4px', '0'),
    backgroundColor: tokens.colorNeutralBackground4,
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
    borderLeftWidth: '0',
    ':hover': {
      backgroundColor: tokens.colorBrandBackground2,
    },
  },
  header: {
    ...shorthands.padding('16px'),
    ...shorthands.borderBottom('1px', 'solid', tokens.colorNeutralStroke1),
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
    fontFamily: 'var(--font-display)',
    letterSpacing: '-0.01em',
    textTransform: 'uppercase',
    color: tokens.colorNeutralForeground2,
  },
  trackList: {
    flex: 1,
    overflow: 'auto',
    ...shorthands.padding('8px'),
  },
  track: {
    ...shorthands.padding('12px'),
    ...shorthands.margin('4px', '0'),
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.borderRadius('4px'),
    ...shorthands.border('1px', 'solid', 'transparent'),
    cursor: 'pointer',
    ...shorthands.transition('all', '150ms', 'ease'),
    ':hover': {
      backgroundColor: tokens.colorNeutralBackground1Hover,
      borderColor: tokens.colorNeutralStroke2,
    },
  },
  trackSelected: {
    backgroundColor: tokens.colorBrandBackground2,
    borderColor: tokens.colorBrandStroke1,
    boxShadow: '0 2px 8px rgba(0,0,0,0.3)',
  },
  trackName: {
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
    marginBottom: '4px',
    color: tokens.colorNeutralForeground1,
  },
  trackInfo: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground3,
  },
});

export function Sidebar() {
  const styles = useStyles();
  const {
    sidebarOpen,
    setSidebarOpen,
    selectedTrackId,
    setSelectedTrackId,
  } = useAppStore();
  const project = useProject();

  const tracks = project?.project.composition.tracks || [];

  return (
    <div className={styles.wrapper}>
      <div className={`${styles.sidebar} ${!sidebarOpen ? styles.sidebarCollapsed : ''}`}>
        <div className={styles.header}>Tracks</div>

        <div className={styles.trackList}>
          {tracks.length === 0 ? (
            <div style={{ padding: '16px', color: 'var(--color-charcoal-300)', textAlign: 'center', fontSize: '13px' }}>
              No tracks yet.
              <br />
              Import a Guitar Pro file to get started.
            </div>
          ) : (
            tracks.map((track: any) => (
              <div
                key={track.id}
                className={`${styles.track} ${
                  selectedTrackId === track.id ? styles.trackSelected : ''
                }`}
                onClick={() => setSelectedTrackId(track.id)}
                role="button"
                tabIndex={0}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    setSelectedTrackId(track.id);
                  }
                }}
              >
                <div className={styles.trackName}>{track.name}</div>
                <div className={styles.trackInfo}>
                  {track.instrument?.type || 'Unknown'} •{' '}
                  {track.instrument?.tuning?.join('-') || 'Standard'}
                </div>
              </div>
            ))
          )}
        </div>
      </div>

      <Button
        className={styles.toggleButton}
        icon={sidebarOpen ? <ChevronLeft24Regular /> : <ChevronRight24Regular />}
        appearance="subtle"
        onClick={() => setSidebarOpen(!sidebarOpen)}
        title={sidebarOpen ? 'Collapse sidebar' : 'Expand sidebar'}
      />
    </div>
  );
}
