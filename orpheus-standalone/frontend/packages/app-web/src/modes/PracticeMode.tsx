/**
 * Practice Mode - Speed trainer and learning tools
 */

import { makeStyles, shorthands, tokens, Button, Card } from '@fluentui/react-components';
import { BotRegular, FolderOpen24Regular, Play24Regular, Stop24Regular } from '@fluentui/react-icons';
import { useState, useEffect, useCallback } from 'react';
import { useAppStore, useProject } from '../store/app-store';
import { SpeedTrainer } from '../components/SpeedTrainer';
import { importFile } from '../services/file-import';
import { getPlaybackCoordinator } from '../services/playback-coordinator';

const useStyles = makeStyles({
  container: {
    height: '100%',
    display: 'flex',
    flexDirection: 'column',
  },
  header: {
    ...shorthands.padding('16px'),
    ...shorthands.borderBottom('1px', 'solid', tokens.colorNeutralStroke1),
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  title: {
    fontSize: '20px',
    fontWeight: tokens.fontWeightSemibold,
  },
  content: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    ...shorthands.padding('24px'),
    ...shorthands.gap('24px'),
  },
  description: {
    textAlign: 'center',
    maxWidth: '600px',
    color: tokens.colorNeutralForeground2,
  },
  trainerContainer: {
    width: '100%',
    maxWidth: '800px',
  },
  emptyState: {
    flex: 1,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexDirection: 'column',
    ...shorthands.gap('16px'),
  },
  projectInfo: {
    width: '100%',
    maxWidth: '800px',
    marginBottom: '16px',
  },
  projectCard: {
    ...shorthands.padding('16px'),
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  projectDetails: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('4px'),
  },
  projectTitle: {
    fontSize: '16px',
    fontWeight: tokens.fontWeightSemibold,
  },
  projectMeta: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground2,
  },
  playbackControls: {
    display: 'flex',
    ...shorthands.gap('8px'),
  },
});

export function PracticeMode() {
  const styles = useStyles();
  const project = useProject();
  const { setAIAssistantOpen, setProject, setMode } = useAppStore();
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTempo, setCurrentTempo] = useState(120);
  const [initialized, setInitialized] = useState(false);

  const coordinator = getPlaybackCoordinator();

  // Initialize playback coordinator with project
  useEffect(() => {
    const initCoordinator = async () => {
      if (project && !initialized) {
        try {
          await coordinator.initialize(project);
          setCurrentTempo(project.project.metadata.tempo || 120);
          setInitialized(true);
          console.log('[PracticeMode] Playback coordinator initialized');
        } catch (error) {
          console.error('[PracticeMode] Failed to initialize coordinator:', error);
        }
      }
    };

    initCoordinator();
  }, [project, coordinator, initialized]);

  // Subscribe to playback state changes
  useEffect(() => {
    const unsubscribe = coordinator.onStateChange((state) => {
      setIsPlaying(state.isPlaying);
    });

    return () => {
      unsubscribe();
    };
  }, [coordinator]);

  const handleSpeedChange = useCallback((bpm: number) => {
    setCurrentTempo(bpm);
    coordinator.setTempo(bpm);
    console.log(`[PracticeMode] Tempo changed to ${bpm} BPM`);
  }, [coordinator]);

  const handlePlayPause = async () => {
    if (isPlaying) {
      coordinator.pause();
    } else {
      await coordinator.play();
    }
  };

  const handleStop = () => {
    coordinator.stop();
  };

  const handleImportFile = async () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.gp,.gpx,.gp5,.gp4,.gp3,.maestro';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        const result = await importFile(file);
        if (result.success && result.project) {
          setProject(result.project);
          setInitialized(false); // Reinitialize coordinator with new project
        }
      }
    };
    input.click();
  };

  if (!project) {
    return (
      <div className={styles.container}>
        <div className={styles.header}>
          <div className={styles.title}>🎸 Practice - Learning Tools</div>
        </div>
        <div className={styles.emptyState}>
          <h3>No Project Loaded</h3>
          <p style={{ color: tokens.colorNeutralForeground2, marginBottom: '24px' }}>
            To practice with your music, load a project first.
          </p>
          <div style={{ display: 'flex', gap: '12px', justifyContent: 'center' }}>
            <Button
              appearance="primary"
              icon={<FolderOpen24Regular />}
              onClick={handleImportFile}
            >
              Import Guitar Pro File
            </Button>
            <Button
              appearance="secondary"
              onClick={() => setMode('compose')}
            >
              Go to Compose Mode
            </Button>
          </div>
          <p style={{ fontSize: '12px', color: tokens.colorNeutralForeground3, marginTop: '16px' }}>
            Or press <strong>Ctrl+1</strong> to switch to Compose mode
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.title}>🎸 Practice - Learning Tools</div>
        <Button icon={<BotRegular />} appearance="subtle" onClick={() => setAIAssistantOpen(true)}>
          AI Practice Coach
        </Button>
      </div>
      <div className={styles.content}>
        {/* Project Info Card */}
        <div className={styles.projectInfo}>
          <Card className={styles.projectCard}>
            <div className={styles.projectDetails}>
              <div className={styles.projectTitle}>
                {project.project.metadata.title || 'Untitled'}
              </div>
              <div className={styles.projectMeta}>
                {project.project.metadata.artist && `${project.project.metadata.artist} • `}
                Original tempo: {project.project.metadata.tempo || 120} BPM
                {project.project.composition?.tracks &&
                  ` • ${project.project.composition.tracks.length} tracks`}
              </div>
            </div>
            <div className={styles.playbackControls}>
              <Button
                icon={isPlaying ? <Stop24Regular /> : <Play24Regular />}
                appearance={isPlaying ? 'secondary' : 'primary'}
                onClick={handlePlayPause}
              >
                {isPlaying ? 'Stop' : 'Play'}
              </Button>
            </div>
          </Card>
        </div>

        <div className={styles.description}>
          <h2>Speed Trainer & Progressive Practice</h2>
          <p style={{ marginTop: '12px', lineHeight: '1.6' }}>
            Build speed gradually with automatic tempo increments.
            <br />
            Practice slowly and perfectly, then watch your speed increase!
          </p>
        </div>

        <div className={styles.trainerContainer}>
          <SpeedTrainer
            initialBPM={Math.floor((project.project.metadata.tempo || 120) * 0.5)}
            targetBPM={project.project.metadata.tempo || 120}
            incrementStep={5}
            repsBeforeIncrement={3}
            onSpeedChange={handleSpeedChange}
          />
        </div>
      </div>
    </div>
  );
}
