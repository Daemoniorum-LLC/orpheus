/**
 * AI Quick Actions Component
 * Provides context-aware quick action buttons for common AI tasks
 * ENHANCED: Now includes smart project state detection and dynamic suggestions
 */

import {
  makeStyles,
  shorthands,
  tokens,
  Button,
} from '@fluentui/react-components';
import {
  Lightbulb24Regular,
  MusicNote224Regular,
  BookInformation24Regular,
  SlideText24Regular,
  Warning24Regular,
} from '@fluentui/react-icons';
import { useCurrentMode, useAppStore } from '../store/app-store';
import type { AppMode } from '../store/app-store';
import { useMemo } from 'react';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
  },
  sectionTitle: {
    fontSize: '12px',
    fontWeight: tokens.fontWeightSemibold,
    color: tokens.colorNeutralForeground3,
    textTransform: 'uppercase',
    marginBottom: '4px',
  },
  quickActions: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('6px'),
  },
  actionButton: {
    justifyContent: 'flex-start',
    ...shorthands.padding('10px', '12px'),
    height: 'auto',
  },
  actionContent: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'flex-start',
    textAlign: 'left',
    ...shorthands.gap('2px'),
  },
  actionTitle: {
    fontSize: '13px',
    fontWeight: 600,
  },
  actionDesc: {
    fontSize: '11px',
    color: tokens.colorNeutralForeground3,
    lineHeight: '1.3',
  },
  smartSuggestion: {
    ...shorthands.padding('12px'),
    backgroundColor: tokens.colorPaletteLightGreenBackground2,
    ...shorthands.borderRadius('8px'),
    ...shorthands.border('1px', 'solid', tokens.colorPaletteLightGreenBorder2),
    marginBottom: '8px',
  },
  smartIcon: {
    color: tokens.colorPaletteLightGreenForeground1,
  },
  smartTitle: {
    fontSize: '12px',
    fontWeight: 600,
    color: tokens.colorPaletteLightGreenForeground2,
    marginBottom: '4px',
  },
  smartText: {
    fontSize: '11px',
    color: tokens.colorNeutralForeground2,
    lineHeight: '1.4',
  },
});

export interface QuickAction {
  id: string;
  title: string;
  description: string;
  prompt: string;
  icon: JSX.Element;
}

/**
 * Smart Context Detection
 * Analyzes project state and suggests relevant actions
 */
interface SmartSuggestion {
  title: string;
  message: string;
  actions: QuickAction[];
}

function getSmartSuggestions(mode: AppMode, projectState: any): SmartSuggestion | null {
  const { project } = projectState;

  // No project loaded - suggest getting started
  if (!project) {
    return {
      title: '🚀 Get Started',
      message: 'No project loaded yet. I can help you start a new composition or explain how Maestro works!',
      actions: [
        {
          id: 'how-to-start',
          title: 'How do I get started?',
          description: 'Learn the basics of Maestro',
          prompt: 'I\'m new to Maestro AI. Can you explain how to get started with a new project?',
          icon: <BookInformation24Regular />,
        },
      ],
    };
  }

  // Get track count from project if available
  const trackCount = project?.project?.tracks?.length || 0;

  // Mode-specific smart detection
  switch (mode) {
    case 'compose':
      // Detect if project has few tracks - suggest composition help
      if (trackCount < 2) {
        return {
          title: '🎼 Composition Tips',
          message: 'You\'re just getting started with this composition. I can help with chord progressions, melody ideas, and song structure!',
          actions: [
            {
              id: 'composition-start',
              title: 'Help me start composing',
              description: 'Get composition guidance',
              prompt: 'I\'m starting a new composition. Can you help me with chord progressions and structure for a song in [key] with a [genre] feel?',
              icon: <Lightbulb24Regular />,
            },
          ],
        };
      }
      break;

    case 'master':
      // This will be enhanced when we have LUFS data
      // For now, suggest loudness targets
      return {
        title: '✨ Mastering Guidance',
        message: 'Getting your master ready for release? I can help you hit the right LUFS targets for different platforms!',
        actions: [
          {
            id: 'check-lufs',
            title: 'Check LUFS Targets',
            description: 'Ensure proper loudness',
            prompt: 'Can you explain the LUFS targets for Spotify, Apple Music, and YouTube? How do I master for streaming?',
            icon: <Warning24Regular />,
          },
        ],
      };

    case 'practice':
      // Suggest practice routine
      return {
        title: '🎸 Practice Smart',
        message: 'Effective practice is key to improvement. I can help you create a structured practice routine!',
        actions: [
          {
            id: 'build-routine',
            title: 'Build Practice Routine',
            description: 'Create a focused practice plan',
            prompt: 'Can you help me create an effective practice routine? I want to work on [technique/skill] and I have about [time] minutes per day.',
            icon: <SlideText24Regular />,
          },
        ],
      };

    case 'distribute':
      // Suggest release preparation
      return {
        title: '🌍 Release Preparation',
        message: 'Ready to release your music? I can guide you through the distribution process!',
        actions: [
          {
            id: 'release-prep',
            title: 'Prepare for Release',
            description: 'Complete release checklist',
            prompt: 'I want to release my music to streaming platforms. Can you walk me through the complete checklist and best practices?',
            icon: <SlideText24Regular />,
          },
        ],
      };
  }

  return null;
}

const QUICK_ACTIONS: Record<AppMode, QuickAction[]> = {
  compose: [
    {
      id: 'suggest-progression',
      title: 'Suggest Chord Progression',
      description: 'Get AI-powered chord progression ideas',
      prompt: 'Can you suggest some chord progressions for my composition? I\'m working in [key] and going for a [genre] vibe.',
      icon: <MusicNote224Regular />,
    },
    {
      id: 'analyze-theory',
      title: 'Analyze Music Theory',
      description: 'Understand the theory behind your music',
      prompt: 'Can you analyze the music theory in my current composition? What scales and chord progressions am I using?',
      icon: <BookInformation24Regular />,
    },
    {
      id: 'improve-melody',
      title: 'Improve My Melody',
      description: 'Get suggestions to enhance your melody',
      prompt: 'How can I improve the melody I\'m working on? Any suggestions for making it more memorable?',
      icon: <Lightbulb24Regular />,
    },
    {
      id: 'song-structure',
      title: 'Help with Song Structure',
      description: 'Get guidance on song arrangement',
      prompt: 'I need help with song structure. What\'s a good arrangement for a [genre] song?',
      icon: <SlideText24Regular />,
    },
  ],
  record: [
    {
      id: 'mic-placement',
      title: 'Microphone Placement',
      description: 'Get tips on mic positioning',
      prompt: 'What\'s the best microphone placement for recording [instrument]?',
      icon: <Lightbulb24Regular />,
    },
    {
      id: 'gain-staging',
      title: 'Check My Gain Staging',
      description: 'Ensure proper recording levels',
      prompt: 'Can you guide me through proper gain staging for my recording session?',
      icon: <BookInformation24Regular />,
    },
    {
      id: 'recording-tips',
      title: 'Recording Tips',
      description: 'Get performance and capture advice',
      prompt: 'What are some tips for getting the best recording performance for [instrument/vocals]?',
      icon: <MusicNote224Regular />,
    },
    {
      id: 'troubleshoot',
      title: 'Troubleshoot Issue',
      description: 'Fix recording problems',
      prompt: 'I\'m having an issue with my recording: [describe problem]. Can you help troubleshoot?',
      icon: <Lightbulb24Regular />,
    },
  ],
  mix: [
    {
      id: 'eq-guidance',
      title: 'EQ Guidance',
      description: 'Get EQ advice for your mix',
      prompt: 'How should I EQ my [instrument] in this mix? It sounds [describe issue].',
      icon: <Lightbulb24Regular />,
    },
    {
      id: 'compression-tips',
      title: 'Compression Tips',
      description: 'Learn proper compression techniques',
      prompt: 'Can you explain compression settings for [instrument]? What ratio, attack, and release should I use?',
      icon: <BookInformation24Regular />,
    },
    {
      id: 'mix-balance',
      title: 'Check Mix Balance',
      description: 'Analyze overall mix balance',
      prompt: 'Can you analyze my mix balance? Does everything sit well together?',
      icon: <MusicNote224Regular />,
    },
    {
      id: 'fix-muddy',
      title: 'Fix Muddy Mix',
      description: 'Clear up frequency buildup',
      prompt: 'My mix sounds muddy. How can I clean it up and get more clarity?',
      icon: <Lightbulb24Regular />,
    },
  ],
  master: [
    {
      id: 'loudness-target',
      title: 'Loudness Targets',
      description: 'Check LUFS for streaming',
      prompt: 'What are the optimal LUFS targets for [platform]? How should I master for streaming?',
      icon: <Lightbulb24Regular />,
    },
    {
      id: 'mastering-chain',
      title: 'Mastering Chain',
      description: 'Build a mastering signal flow',
      prompt: 'What should my mastering chain look like? What order should I process my master bus?',
      icon: <SlideText24Regular />,
    },
    {
      id: 'reference-match',
      title: 'Match Reference Track',
      description: 'Compare to professional masters',
      prompt: 'How can I match the loudness and tone of my reference track while mastering?',
      icon: <MusicNote224Regular />,
    },
    {
      id: 'format-export',
      title: 'Export Format',
      description: 'Choose correct export settings',
      prompt: 'What format and settings should I export for [platform/use case]?',
      icon: <BookInformation24Regular />,
    },
  ],
  practice: [
    {
      id: 'practice-routine',
      title: 'Create Practice Routine',
      description: 'Build a structured practice plan',
      prompt: 'Can you create a practice routine for improving [technique/skill]? I have [time] minutes per day.',
      icon: <SlideText24Regular />,
    },
    {
      id: 'speed-building',
      title: 'Speed Building',
      description: 'Techniques for playing faster',
      prompt: 'How can I build speed on this riff? What\'s the best practice approach?',
      icon: <Lightbulb24Regular />,
    },
    {
      id: 'technique-help',
      title: 'Technique Help',
      description: 'Master a specific technique',
      prompt: 'I\'m struggling with [technique]. Can you break it down and give me exercises?',
      icon: <BookInformation24Regular />,
    },
    {
      id: 'song-learning',
      title: 'Learn This Song',
      description: 'Strategy for learning songs',
      prompt: 'What\'s the best approach to learn this song? Should I break it into sections?',
      icon: <MusicNote224Regular />,
    },
  ],
  distribute: [
    {
      id: 'release-checklist',
      title: 'Release Checklist',
      description: 'Ensure everything is ready',
      prompt: 'What\'s the complete checklist for releasing music to streaming platforms?',
      icon: <SlideText24Regular />,
    },
    {
      id: 'metadata-help',
      title: 'Metadata Optimization',
      description: 'Fill out track information',
      prompt: 'Can you help me optimize my metadata for better discoverability on streaming platforms?',
      icon: <BookInformation24Regular />,
    },
    {
      id: 'platform-comparison',
      title: 'Compare Distributors',
      description: 'Choose the right distributor',
      prompt: 'What are the pros and cons of different music distributors? Which one should I use?',
      icon: <Lightbulb24Regular />,
    },
    {
      id: 'release-strategy',
      title: 'Release Strategy',
      description: 'Plan your release timing',
      prompt: 'What\'s a good release strategy for my music? How far in advance should I submit?',
      icon: <MusicNote224Regular />,
    },
  ],
};

export interface AIQuickActionsProps {
  onActionClick: (prompt: string) => void;
}

export function AIQuickActions({ onActionClick }: AIQuickActionsProps) {
  const styles = useStyles();
  const mode = useCurrentMode();
  const { project } = useAppStore();

  // Get smart suggestions based on current state
  const smartSuggestion = useMemo(() => {
    return getSmartSuggestions(mode, { project });
  }, [mode, project]);

  // Get mode-specific quick actions
  const actions = QUICK_ACTIONS[mode] || [];

  // Show smart suggestion if available, otherwise show standard quick actions
  const displayActions = smartSuggestion?.actions || actions;

  if (displayActions.length === 0) return null;

  return (
    <div className={styles.container}>
      {/* Smart Suggestion Highlight */}
      {smartSuggestion && (
        <div className={styles.smartSuggestion}>
          <div className={styles.smartTitle}>
            {smartSuggestion.title}
          </div>
          <div className={styles.smartText}>
            {smartSuggestion.message}
          </div>
        </div>
      )}

      {/* Quick Actions */}
      <div className={styles.sectionTitle}>
        {smartSuggestion ? '💡 Suggested Actions' : '✨ Quick Actions'}
      </div>
      <div className={styles.quickActions}>
        {displayActions.map((action) => (
          <Button
            key={action.id}
            appearance="subtle"
            icon={action.icon}
            className={styles.actionButton}
            onClick={() => onActionClick(action.prompt)}
          >
            <div className={styles.actionContent}>
              <div className={styles.actionTitle}>{action.title}</div>
              <div className={styles.actionDesc}>{action.description}</div>
            </div>
          </Button>
        ))}

        {/* If we have smart suggestions, also show first 2 regular actions */}
        {smartSuggestion && actions.slice(0, 2).map((action) => (
          <Button
            key={action.id}
            appearance="subtle"
            icon={action.icon}
            className={styles.actionButton}
            onClick={() => onActionClick(action.prompt)}
          >
            <div className={styles.actionContent}>
              <div className={styles.actionTitle}>{action.title}</div>
              <div className={styles.actionDesc}>{action.description}</div>
            </div>
          </Button>
        ))}
      </div>
    </div>
  );
}
