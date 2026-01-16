/**
 * Input Level Meter - Real-time audio input level indicator
 */

import { makeStyles, shorthands, tokens } from '@fluentui/react-components';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
  },
  label: {
    fontSize: '12px',
    fontWeight: tokens.fontWeightSemibold,
    color: tokens.colorNeutralForeground2,
  },
  meterContainer: {
    width: '100%',
    height: '24px',
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.borderRadius('4px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
    overflow: 'hidden',
    position: 'relative',
  },
  meterFill: {
    height: '100%',
    ...shorthands.transition('width', '50ms', 'ease-out'),
  },
  levelText: {
    position: 'absolute',
    top: '50%',
    left: '50%',
    transform: 'translate(-50%, -50%)',
    fontSize: '11px',
    fontWeight: 600,
    color: tokens.colorNeutralForeground1,
    textShadow: '0 0 2px rgba(0, 0, 0, 0.5)',
    pointerEvents: 'none',
  },
});

interface InputLevelMeterProps {
  level: number; // 0-1
  showLabel?: boolean;
}

export function InputLevelMeter({ level, showLabel = true }: InputLevelMeterProps) {
  const styles = useStyles();

  // Determine color based on level
  const getColor = (level: number): string => {
    if (level < 0.3) return tokens.colorPaletteGreenBackground3; // Low - green
    if (level < 0.7) return tokens.colorPaletteYellowBackground3; // Medium - yellow
    if (level < 0.9) return tokens.colorPaletteDarkOrangeBackground3; // High - orange
    return tokens.colorPaletteRedBackground3; // Clipping - red
  };

  const getStatus = (level: number): string => {
    if (level < 0.3) return 'Low';
    if (level < 0.7) return 'Good';
    if (level < 0.9) return 'High';
    return 'CLIPPING!';
  };

  const percentage = Math.round(level * 100);

  return (
    <div className={styles.container}>
      {showLabel && <div className={styles.label}>Input Level: {getStatus(level)}</div>}
      <div className={styles.meterContainer}>
        <div
          className={styles.meterFill}
          style={{
            width: `${percentage}%`,
            backgroundColor: getColor(level),
          }}
        />
        <div className={styles.levelText}>{percentage}%</div>
      </div>
    </div>
  );
}
