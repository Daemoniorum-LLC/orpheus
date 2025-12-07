import { Button } from '@persona-framework/ui';
import {
  Lightbulb,
  Music2,
  BookOpen,
  ListChecks,
  AlertTriangle,
} from 'lucide-react';
import { useCurrentMode, useAppStore } from '../store/app-store';
import type { AppMode } from '../store/app-store';
import { useMemo } from 'react';

export interface QuickAction {
  id: string;
  title: string;
  description: string;
  prompt: string;
  icon: JSX.Element;
}

interface SmartSuggestion {
  title: string;
  message: string;
  actions: QuickAction[];
}

function getSmartSuggestions(mode: AppMode, projectState: any): SmartSuggestion | null {
  const { project } = projectState;

  if (!project) {
    return {
      title: '🚀 Get Started',
      message: 'No project loaded yet. I can help you start a new composition or explain how Orpheus works!',
      actions: [
        {
          id: 'how-to-start',
          title: 'How do I get started?',
          description: 'Learn the basics of Orpheus',
          prompt: 'I\'m new to Orpheus. Can you explain how to get started with a new project?',
          icon: <BookOpen className="h-5 w-5" />,
        },
      ],
    };
  }

  const trackCount = project?.project?.tracks?.length || 0;

  switch (mode) {
    case 'compose':
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
              icon: <Lightbulb className="h-5 w-5" />,
            },
          ],
        };
      }
      break;

    case 'master':
      return {
        title: '✨ Mastering Guidance',
        message: 'Getting your master ready for release? I can help you hit the right LUFS targets for different platforms!',
        actions: [
          {
            id: 'check-lufs',
            title: 'Check LUFS Targets',
            description: 'Ensure proper loudness',
            prompt: 'Can you explain the LUFS targets for Spotify, Apple Music, and YouTube? How do I master for streaming?',
            icon: <AlertTriangle className="h-5 w-5" />,
          },
        ],
      };

    case 'practice':
      return {
        title: '🎸 Practice Smart',
        message: 'Effective practice is key to improvement. I can help you create a structured practice routine!',
        actions: [
          {
            id: 'build-routine',
            title: 'Build Practice Routine',
            description: 'Create a focused practice plan',
            prompt: 'Can you help me create an effective practice routine? I want to work on [technique/skill] and I have about [time] minutes per day.',
            icon: <ListChecks className="h-5 w-5" />,
          },
        ],
      };

    case 'distribute':
      return {
        title: '🌍 Release Preparation',
        message: 'Ready to release your music? I can guide you through the distribution process!',
        actions: [
          {
            id: 'release-prep',
            title: 'Prepare for Release',
            description: 'Complete release checklist',
            prompt: 'I want to release my music to streaming platforms. Can you walk me through the complete checklist and best practices?',
            icon: <ListChecks className="h-5 w-5" />,
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
      icon: <Music2 className="h-5 w-5" />,
    },
    {
      id: 'analyze-theory',
      title: 'Analyze Music Theory',
      description: 'Understand the theory behind your music',
      prompt: 'Can you analyze the music theory in my current composition? What scales and chord progressions am I using?',
      icon: <BookOpen className="h-5 w-5" />,
    },
    {
      id: 'improve-melody',
      title: 'Improve My Melody',
      description: 'Get suggestions to enhance your melody',
      prompt: 'How can I improve the melody I\'m working on? Any suggestions for making it more memorable?',
      icon: <Lightbulb className="h-5 w-5" />,
    },
    {
      id: 'song-structure',
      title: 'Help with Song Structure',
      description: 'Get guidance on song arrangement',
      prompt: 'I need help with song structure. What\'s a good arrangement for a [genre] song?',
      icon: <ListChecks className="h-5 w-5" />,
    },
  ],
  record: [
    {
      id: 'mic-placement',
      title: 'Microphone Placement',
      description: 'Get tips on mic positioning',
      prompt: 'What\'s the best microphone placement for recording [instrument]?',
      icon: <Lightbulb className="h-5 w-5" />,
    },
    {
      id: 'gain-staging',
      title: 'Check My Gain Staging',
      description: 'Ensure proper recording levels',
      prompt: 'Can you guide me through proper gain staging for my recording session?',
      icon: <BookOpen className="h-5 w-5" />,
    },
    {
      id: 'recording-tips',
      title: 'Recording Tips',
      description: 'Get performance and capture advice',
      prompt: 'What are some tips for getting the best recording performance for [instrument/vocals]?',
      icon: <Music2 className="h-5 w-5" />,
    },
    {
      id: 'troubleshoot',
      title: 'Troubleshoot Issue',
      description: 'Fix recording problems',
      prompt: 'I\'m having an issue with my recording: [describe problem]. Can you help troubleshoot?',
      icon: <Lightbulb className="h-5 w-5" />,
    },
  ],
  mix: [
    {
      id: 'eq-guidance',
      title: 'EQ Guidance',
      description: 'Get EQ advice for your mix',
      prompt: 'How should I EQ my [instrument] in this mix? It sounds [describe issue].',
      icon: <Lightbulb className="h-5 w-5" />,
    },
    {
      id: 'compression-tips',
      title: 'Compression Tips',
      description: 'Learn proper compression techniques',
      prompt: 'Can you explain compression settings for [instrument]? What ratio, attack, and release should I use?',
      icon: <BookOpen className="h-5 w-5" />,
    },
    {
      id: 'mix-balance',
      title: 'Check Mix Balance',
      description: 'Analyze overall mix balance',
      prompt: 'Can you analyze my mix balance? Does everything sit well together?',
      icon: <Music2 className="h-5 w-5" />,
    },
    {
      id: 'fix-muddy',
      title: 'Fix Muddy Mix',
      description: 'Clear up frequency buildup',
      prompt: 'My mix sounds muddy. How can I clean it up and get more clarity?',
      icon: <Lightbulb className="h-5 w-5" />,
    },
  ],
  master: [
    {
      id: 'loudness-target',
      title: 'Loudness Targets',
      description: 'Check LUFS for streaming',
      prompt: 'What are the optimal LUFS targets for [platform]? How should I master for streaming?',
      icon: <Lightbulb className="h-5 w-5" />,
    },
    {
      id: 'mastering-chain',
      title: 'Mastering Chain',
      description: 'Build a mastering signal flow',
      prompt: 'What should my mastering chain look like? What order should I process my master bus?',
      icon: <ListChecks className="h-5 w-5" />,
    },
    {
      id: 'reference-match',
      title: 'Match Reference Track',
      description: 'Compare to professional masters',
      prompt: 'How can I match the loudness and tone of my reference track while mastering?',
      icon: <Music2 className="h-5 w-5" />,
    },
    {
      id: 'format-export',
      title: 'Export Format',
      description: 'Choose correct export settings',
      prompt: 'What format and settings should I export for [platform/use case]?',
      icon: <BookOpen className="h-5 w-5" />,
    },
  ],
  practice: [
    {
      id: 'practice-routine',
      title: 'Create Practice Routine',
      description: 'Build a structured practice plan',
      prompt: 'Can you create a practice routine for improving [technique/skill]? I have [time] minutes per day.',
      icon: <ListChecks className="h-5 w-5" />,
    },
    {
      id: 'speed-building',
      title: 'Speed Building',
      description: 'Techniques for playing faster',
      prompt: 'How can I build speed on this riff? What\'s the best practice approach?',
      icon: <Lightbulb className="h-5 w-5" />,
    },
    {
      id: 'technique-help',
      title: 'Technique Help',
      description: 'Master a specific technique',
      prompt: 'I\'m struggling with [technique]. Can you break it down and give me exercises?',
      icon: <BookOpen className="h-5 w-5" />,
    },
    {
      id: 'song-learning',
      title: 'Learn This Song',
      description: 'Strategy for learning songs',
      prompt: 'What\'s the best approach to learn this song? Should I break it into sections?',
      icon: <Music2 className="h-5 w-5" />,
    },
  ],
  distribute: [
    {
      id: 'release-checklist',
      title: 'Release Checklist',
      description: 'Ensure everything is ready',
      prompt: 'What\'s the complete checklist for releasing music to streaming platforms?',
      icon: <ListChecks className="h-5 w-5" />,
    },
    {
      id: 'metadata-help',
      title: 'Metadata Optimization',
      description: 'Fill out track information',
      prompt: 'Can you help me optimize my metadata for better discoverability on streaming platforms?',
      icon: <BookOpen className="h-5 w-5" />,
    },
    {
      id: 'platform-comparison',
      title: 'Compare Distributors',
      description: 'Choose the right distributor',
      prompt: 'What are the pros and cons of different music distributors? Which one should I use?',
      icon: <Lightbulb className="h-5 w-5" />,
    },
    {
      id: 'release-strategy',
      title: 'Release Strategy',
      description: 'Plan your release timing',
      prompt: 'What\'s a good release strategy for my music? How far in advance should I submit?',
      icon: <Music2 className="h-5 w-5" />,
    },
  ],
};

export interface AIQuickActionsProps {
  onActionClick: (prompt: string) => void;
}

export function AIQuickActions({ onActionClick }: AIQuickActionsProps) {
  const mode = useCurrentMode();
  const { project } = useAppStore();

  const smartSuggestion = useMemo(() => {
    return getSmartSuggestions(mode, { project });
  }, [mode, project]);

  const actions = QUICK_ACTIONS[mode] || [];
  const displayActions = smartSuggestion?.actions || actions;

  if (displayActions.length === 0) return null;

  return (
    <div className="flex flex-col gap-2">
      {smartSuggestion && (
        <div className="px-3 py-3 bg-green-500/10 rounded-lg border border-green-500/20 mb-2">
          <div className="text-xs font-semibold text-green-400 mb-1">
            {smartSuggestion.title}
          </div>
          <div className="text-[11px] text-muted-foreground leading-snug">
            {smartSuggestion.message}
          </div>
        </div>
      )}

      <div className="text-xs font-semibold text-muted-foreground uppercase mb-1">
        {smartSuggestion ? '💡 Suggested Actions' : '✨ Quick Actions'}
      </div>
      <div className="flex flex-col gap-1.5">
        {displayActions.map((action) => (
          <Button
            key={action.id}
            variant="ghost"
            className="justify-start h-auto px-3 py-2.5 text-left"
            onClick={() => onActionClick(action.prompt)}
          >
            <div className="flex items-start gap-2 w-full">
              <div className="mt-0.5">{action.icon}</div>
              <div className="flex flex-col gap-0.5 flex-1 min-w-0">
                <div className="text-[13px] font-semibold leading-tight">{action.title}</div>
                <div className="text-[11px] text-muted-foreground leading-tight">
                  {action.description}
                </div>
              </div>
            </div>
          </Button>
        ))}

        {smartSuggestion && actions.slice(0, 2).map((action) => (
          <Button
            key={action.id}
            variant="ghost"
            className="justify-start h-auto px-3 py-2.5 text-left"
            onClick={() => onActionClick(action.prompt)}
          >
            <div className="flex items-start gap-2 w-full">
              <div className="mt-0.5">{action.icon}</div>
              <div className="flex flex-col gap-0.5 flex-1 min-w-0">
                <div className="text-[13px] font-semibold leading-tight">{action.title}</div>
                <div className="text-[11px] text-muted-foreground leading-tight">
                  {action.description}
                </div>
              </div>
            </div>
          </Button>
        ))}
      </div>
    </div>
  );
}
