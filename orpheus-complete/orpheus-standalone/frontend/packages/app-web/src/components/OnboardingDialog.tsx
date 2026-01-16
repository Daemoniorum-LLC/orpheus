/**
 * Onboarding Dialog - First-run tutorial for new users
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
import {
  MusicNote224Regular,
  Record24Regular,
  SpeakerSettings24Regular,
  ArrowRight24Regular,
  Checkmark24Regular,
} from '@fluentui/react-icons';
import { useState } from 'react';

const useStyles = makeStyles({
  content: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('24px'),
    minHeight: '300px',
  },
  stepIndicator: {
    display: 'flex',
    justifyContent: 'center',
    ...shorthands.gap('8px'),
    marginBottom: '16px',
  },
  dot: {
    width: '12px',
    height: '12px',
    ...shorthands.borderRadius('50%'),
    backgroundColor: tokens.colorNeutralStroke1,
    ...shorthands.transition('background-color', '200ms'),
  },
  dotActive: {
    backgroundColor: tokens.colorBrandBackground,
  },
  stepContent: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    textAlign: 'center',
    ...shorthands.gap('24px'),
  },
  icon: {
    fontSize: '64px',
    color: tokens.colorBrandForeground1,
  },
  stepTitle: {
    fontSize: '24px',
    fontWeight: tokens.fontWeightSemibold,
    marginBottom: '8px',
  },
  stepDescription: {
    fontSize: '14px',
    color: tokens.colorNeutralForeground2,
    lineHeight: '1.6',
    maxWidth: '400px',
  },
  featureList: {
    listStyle: 'none',
    ...shorthands.padding(0),
    ...shorthands.margin(0),
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('12px'),
    textAlign: 'left',
  },
  featureItem: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('12px'),
    fontSize: '14px',
  },
});

interface OnboardingDialogProps {
  open: boolean;
  onClose: () => void;
}

const steps = [
  {
    title: 'Welcome to Maestro AI',
    description: 'Your all-in-one music production platform with AI-powered assistance.',
    icon: <MusicNote224Regular />,
    features: [
      'Compose and edit guitar tablature',
      'Record multi-track audio',
      'Mix and master your music',
      'Distribute to streaming platforms',
    ],
  },
  {
    title: '6 Powerful Modes',
    description: 'Switch between modes using Ctrl+1 through Ctrl+6, or click the mode buttons at the top.',
    icon: <Record24Regular />,
    features: [
      'Compose - Tab editor with alphaTab rendering',
      'Record - Multi-track audio recording',
      'Mix - Professional mixing console',
      'Master - AI-powered mastering',
      'Practice - Speed trainer and loops',
      'Distribute - Music distribution',
    ],
  },
  {
    title: 'AI Assistant at Your Service',
    description: 'Press Ctrl+K or click the AI button to get intelligent help with any task.',
    icon: <SpeakerSettings24Regular />,
    features: [
      'Context-aware suggestions for each mode',
      'Quick actions for common tasks',
      'Different AI personas for different modes',
      'Natural language music assistance',
    ],
  },
];

export function OnboardingDialog({ open, onClose }: OnboardingDialogProps) {
  const styles = useStyles();
  const [currentStep, setCurrentStep] = useState(0);

  const handleNext = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      // Mark onboarding as complete
      localStorage.setItem('maestro-onboarding-complete', 'true');
      onClose();
    }
  };

  const handleSkip = () => {
    localStorage.setItem('maestro-onboarding-complete', 'true');
    onClose();
  };

  const step = steps[currentStep];
  const isLastStep = currentStep === steps.length - 1;

  return (
    <Dialog open={open} onOpenChange={(_, data) => !data.open && handleSkip()}>
      <DialogSurface style={{ maxWidth: '600px' }}>
        <DialogBody>
          <DialogTitle>
            <div className={styles.stepIndicator}>
              {steps.map((_, index) => (
                <div
                  key={index}
                  className={`${styles.dot} ${index <= currentStep ? styles.dotActive : ''}`}
                />
              ))}
            </div>
          </DialogTitle>
          <DialogContent>
            <div className={styles.content}>
              <div className={styles.stepContent}>
                <div className={styles.icon}>{step.icon}</div>
                <div>
                  <div className={styles.stepTitle}>{step.title}</div>
                  <div className={styles.stepDescription}>{step.description}</div>
                </div>
                <ul className={styles.featureList}>
                  {step.features.map((feature, index) => (
                    <li key={index} className={styles.featureItem}>
                      <Checkmark24Regular style={{ color: tokens.colorPaletteGreenForeground1 }} />
                      <span>{feature}</span>
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          </DialogContent>
          <DialogActions>
            <Button appearance="secondary" onClick={handleSkip}>
              Skip Tutorial
            </Button>
            <Button appearance="primary" onClick={handleNext} icon={isLastStep ? undefined : <ArrowRight24Regular />}>
              {isLastStep ? "Let's Go!" : 'Next'}
            </Button>
          </DialogActions>
        </DialogBody>
      </DialogSurface>
    </Dialog>
  );
}
