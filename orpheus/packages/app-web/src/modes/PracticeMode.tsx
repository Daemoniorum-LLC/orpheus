/**
 * Practice Mode - Speed trainer and learning tools
 */

import { makeStyles, shorthands, tokens, Button } from '@fluentui/react-components';
import { BotRegular } from '@fluentui/react-icons';
import { useAppStore } from '../store/app-store';
import { SpeedTrainer } from '../components/SpeedTrainer';

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
});

export function PracticeMode() {
  const styles = useStyles();
  const { setAIAssistantOpen } = useAppStore();

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.title}>🎸 Practice - Learning Tools</div>
        <Button icon={<BotRegular />} appearance="subtle" onClick={() => setAIAssistantOpen(true)}>
          AI Practice Coach
        </Button>
      </div>
      <div className={styles.content}>
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
            initialBPM={60}
            targetBPM={120}
            incrementStep={5}
            repsBeforeIncrement={3}
          />
        </div>
      </div>
    </div>
  );
}
