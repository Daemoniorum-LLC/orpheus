/**
 * Skeleton Loader - Animated placeholder for loading content
 * Memoized for performance during lazy loading
 */

import { memo } from 'react';
import { makeStyles, shorthands, tokens } from '@fluentui/react-components';

const useStyles = makeStyles({
  skeleton: {
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.borderRadius('4px'),
    position: 'relative',
    overflow: 'hidden',
    '::before': {
      content: '""',
      position: 'absolute',
      top: 0,
      left: '-100%',
      height: '100%',
      width: '100%',
      background: `linear-gradient(90deg, transparent, ${tokens.colorNeutralBackground1}, transparent)`,
      animationName: 'shimmer',
      animationDuration: '1.5s',
      animationIterationCount: 'infinite',
    },
    '@keyframes shimmer': {
      '0%': { left: '-100%' },
      '100%': { left: '100%' },
    },
  },
  container: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('16px'),
    ...shorthands.padding('24px'),
    height: '100%',
  },
  header: {
    height: '40px',
  },
  toolbar: {
    display: 'flex',
    ...shorthands.gap('12px'),
  },
  button: {
    height: '32px',
    width: '100px',
  },
  card: {
    height: '200px',
    ...shorthands.borderRadius('8px'),
  },
  cardSmall: {
    height: '120px',
    ...shorthands.borderRadius('8px'),
  },
  text: {
    height: '16px',
    width: '60%',
  },
  textFull: {
    height: '16px',
    width: '100%',
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(250px, 1fr))',
    ...shorthands.gap('16px'),
  },
});

export interface SkeletonLoaderProps {
  /** Variant of skeleton to show */
  variant?: 'compose' | 'record' | 'mix' | 'master' | 'practice' | 'distribute' | 'generic';
}

export const SkeletonLoader = memo(function SkeletonLoader({ variant = 'generic' }: SkeletonLoaderProps) {
  const styles = useStyles();

  switch (variant) {
    case 'compose':
      return <ComposeSkeleton styles={styles} />;
    case 'record':
      return <RecordSkeleton styles={styles} />;
    case 'mix':
      return <MixSkeleton styles={styles} />;
    case 'master':
      return <MasterSkeleton styles={styles} />;
    case 'practice':
      return <PracticeSkeleton styles={styles} />;
    case 'distribute':
      return <DistributeSkeleton styles={styles} />;
    default:
      return <GenericSkeleton styles={styles} />;
  }
});

function ComposeSkeleton({ styles }: { styles: ReturnType<typeof useStyles> }) {
  return (
    <div className={styles.container}>
      {/* Toolbar */}
      <div className={styles.toolbar}>
        <div className={`${styles.skeleton} ${styles.button}`} />
        <div className={`${styles.skeleton} ${styles.button}`} />
        <div className={`${styles.skeleton} ${styles.button}`} />
      </div>
      {/* Tab Editor */}
      <div className={`${styles.skeleton} ${styles.card}`} style={{ flex: 1 }} />
      {/* Controls */}
      <div className={styles.toolbar}>
        <div className={`${styles.skeleton} ${styles.button}`} />
        <div className={`${styles.skeleton} ${styles.button}`} />
      </div>
    </div>
  );
}

function RecordSkeleton({ styles }: { styles: ReturnType<typeof useStyles> }) {
  return (
    <div className={styles.container}>
      {/* Header */}
      <div className={`${styles.skeleton} ${styles.header}`} />
      {/* Waveform */}
      <div className={`${styles.skeleton} ${styles.card}`} />
      {/* Tracks */}
      <div className={`${styles.skeleton} ${styles.cardSmall}`} />
      <div className={`${styles.skeleton} ${styles.cardSmall}`} />
    </div>
  );
}

function MixSkeleton({ styles }: { styles: ReturnType<typeof useStyles> }) {
  return (
    <div className={styles.container}>
      {/* Mixer Channels */}
      <div className={styles.grid}>
        <div className={`${styles.skeleton} ${styles.card}`} />
        <div className={`${styles.skeleton} ${styles.card}`} />
        <div className={`${styles.skeleton} ${styles.card}`} />
        <div className={`${styles.skeleton} ${styles.card}`} />
      </div>
    </div>
  );
}

function MasterSkeleton({ styles }: { styles: ReturnType<typeof useStyles> }) {
  return (
    <div className={styles.container}>
      {/* Meters */}
      <div className={`${styles.skeleton} ${styles.card}`} />
      {/* Mastering Chain */}
      <div className={`${styles.skeleton} ${styles.card}`} />
      {/* Export Options */}
      <div className={`${styles.skeleton} ${styles.cardSmall}`} />
    </div>
  );
}

function PracticeSkeleton({ styles }: { styles: ReturnType<typeof useStyles> }) {
  return (
    <div className={styles.container}>
      {/* Speed Trainer */}
      <div className={`${styles.skeleton} ${styles.card}`} />
      {/* Loop Section */}
      <div className={`${styles.skeleton} ${styles.cardSmall}`} />
      {/* Progress */}
      <div className={`${styles.skeleton} ${styles.cardSmall}`} />
    </div>
  );
}

function DistributeSkeleton({ styles }: { styles: ReturnType<typeof useStyles> }) {
  return (
    <div className={styles.container}>
      {/* Platform Cards */}
      <div className={styles.grid}>
        <div className={`${styles.skeleton} ${styles.cardSmall}`} />
        <div className={`${styles.skeleton} ${styles.cardSmall}`} />
        <div className={`${styles.skeleton} ${styles.cardSmall}`} />
      </div>
    </div>
  );
}

function GenericSkeleton({ styles }: { styles: ReturnType<typeof useStyles> }) {
  return (
    <div className={styles.container}>
      <div className={`${styles.skeleton} ${styles.text}`} />
      <div className={`${styles.skeleton} ${styles.textFull}`} />
      <div className={`${styles.skeleton} ${styles.card}`} />
      <div className={`${styles.skeleton} ${styles.cardSmall}`} />
    </div>
  );
}
