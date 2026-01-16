/**
 * Mode Selector - Switch between the 5 modes of Maestro AI
 */

import {
  Button,
  Tooltip,
  makeStyles,
  shorthands,
  tokens,
} from '@fluentui/react-components';
import {
  MusicNote224Regular,
  Record24Regular,
  SpeakerSettings24Regular,
  WandRegular,
  LearningApp24Regular,
  CloudArrowUp24Regular,
} from '@fluentui/react-icons';
import { useAppStore, useProject, type AppMode } from '../store/app-store';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    ...shorthands.gap('8px'),
    ...shorthands.padding('12px'),
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.borderBottom('1px', 'solid', tokens.colorNeutralStroke1),
  },
  modeButton: {
    minWidth: '120px',
    height: '48px',
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
  },
  active: {
    backgroundColor: tokens.colorBrandBackground,
    color: tokens.colorNeutralForegroundOnBrand,
    ':hover': {
      backgroundColor: tokens.colorBrandBackgroundHover,
    },
  },
  disabledBadge: {
    fontSize: '10px',
    color: tokens.colorPaletteRedForeground1,
    marginLeft: '4px',
  },
});

interface ModeInfo {
  id: AppMode;
  label: string;
  icon: React.ReactElement;
  description: string;
  shortcut: string;
  requiresProject?: boolean; // New field
}

const modes: ModeInfo[] = [
  {
    id: 'compose',
    label: 'Compose',
    icon: <MusicNote224Regular />,
    description: 'Tablature and notation editing (Cadenza AI)',
    shortcut: 'Ctrl+1',
    requiresProject: false,
  },
  {
    id: 'record',
    label: 'Record',
    icon: <Record24Regular />,
    description: 'Multi-track audio recording (Nexus DAW)',
    shortcut: 'Ctrl+2',
    requiresProject: false,
  },
  {
    id: 'mix',
    label: 'Mix',
    icon: <SpeakerSettings24Regular />,
    description: 'Professional mixing console (Nexus DAW)',
    shortcut: 'Ctrl+3',
    requiresProject: true,
  },
  {
    id: 'master',
    label: 'Master',
    icon: <WandRegular />,
    description: 'AI-powered mastering (Nexus DAW)',
    shortcut: 'Ctrl+4',
    requiresProject: true,
  },
  {
    id: 'practice',
    label: 'Practice',
    icon: <LearningApp24Regular />,
    description: 'Speed trainer and learning tools',
    shortcut: 'Ctrl+5',
    requiresProject: false,
  },
  {
    id: 'distribute',
    label: 'Distribute',
    icon: <CloudArrowUp24Regular />,
    description: 'Music distribution to streaming platforms',
    shortcut: 'Ctrl+6',
    requiresProject: true,
  },
];

export function ModeSelector() {
  const styles = useStyles();
  const { mode, setMode } = useAppStore();
  const project = useProject();

  const handleModeChange = (newMode: AppMode) => {
    setMode(newMode);
  };

  return (
    <div className={styles.container}>
      {modes.map((modeInfo) => {
        const isDisabled = modeInfo.requiresProject && !project;
        const tooltipContent = isDisabled
          ? `${modeInfo.description} - Requires a project to be loaded. Import or create a project in Compose mode first.`
          : `${modeInfo.description} (${modeInfo.shortcut})`;

        return (
          <Tooltip
            key={modeInfo.id}
            content={tooltipContent}
            relationship="label"
          >
            <Button
              className={`${styles.modeButton} ${
                mode === modeInfo.id ? styles.active : ''
              }`}
              appearance={mode === modeInfo.id ? 'primary' : 'secondary'}
              icon={modeInfo.icon}
              onClick={() => handleModeChange(modeInfo.id)}
              disabled={isDisabled}
            >
              {modeInfo.label}
              {isDisabled && <span className={styles.disabledBadge}>⚠</span>}
            </Button>
          </Tooltip>
        );
      })}
    </div>
  );
}
