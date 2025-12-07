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
  sidebar: {
    width: '280px',
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.borderRight('1px', 'solid', tokens.colorNeutralStroke1),
    display: 'flex',
    flexDirection: 'column',
    position: 'relative',
  },
  sidebarCollapsed: {
    width: '0',
    overflow: 'hidden',
  },
  toggleButton: {
    position: 'absolute',
    right: '-16px',
    top: '50%',
    transform: 'translateY(-50%)',
    zIndex: 10,
    borderRadius: '50%',
  },
  header: {
    ...shorthands.padding('16px'),
    ...shorthands.borderBottom('1px', 'solid', tokens.colorNeutralStroke1),
    fontSize: '16px',
    fontWeight: tokens.fontWeightSemibold,
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
    cursor: 'pointer',
    ':hover': {
      backgroundColor: tokens.colorNeutralBackground1Hover,
    },
  },
  trackSelected: {
    backgroundColor: tokens.colorBrandBackground2,
  },
  trackName: {
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
    marginBottom: '4px',
  },
  trackInfo: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground2,
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
    <>
      <div className={`${styles.sidebar} ${!sidebarOpen ? styles.sidebarCollapsed : ''}`}>
        <div className={styles.header}>Tracks</div>

        <div className={styles.trackList}>
          {tracks.length === 0 ? (
            <div style={{ padding: '16px', color: 'var(--color-charcoal-300)', textAlign: 'center' }}>
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

        <Button
          className={styles.toggleButton}
          icon={sidebarOpen ? <ChevronLeft24Regular /> : <ChevronRight24Regular />}
          appearance="subtle"
          onClick={() => setSidebarOpen(!sidebarOpen)}
        />
      </div>
    </>
  );
}
