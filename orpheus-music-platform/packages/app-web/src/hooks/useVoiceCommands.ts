/**
 * Voice Commands Hook - Registers and manages voice commands for Orpheus
 * Connects voice recognition to app actions
 */

import { useEffect, useState, useCallback } from 'react';
import {
  getVoiceCommandService,
  type VoiceCommand,
  type VoiceState,
  type VoiceStateChange,
} from '../services/voice-commands';
import { useAppStore } from '../store/app-store';
import { getPlaybackCoordinator } from '../services/playback-coordinator';
import { saveProject } from '../services/project-save';
import { showSuccess, showInfo, showError } from '../services/toast';
import { loadPreferences } from '../components/PreferencesDialog';
import { getEnhancedCommands } from '../services/voice-commands-enhanced';
import { getMusicVoiceCommands } from '../services/voice-commands-music';
import { getAudioFeedback } from '../services/audio-feedback';

/**
 * Hook to manage voice command state and UI updates
 */
export function useVoiceCommandState() {
  const [state, setState] = useState<VoiceState>('idle');
  const [transcript, setTranscript] = useState<string>('');
  const [matchedCommand, setMatchedCommand] = useState<VoiceCommand | null>(null);
  const [error, setError] = useState<string>('');
  const [isSupported, setIsSupported] = useState(false);

  useEffect(() => {
    const service = getVoiceCommandService();
    setIsSupported(service.isSupported());

    // Apply preferences
    const prefs = loadPreferences();
    service.setWakeWordEnabled(prefs.voiceWakeWordEnabled);

    const unsubscribe = service.subscribe((change: VoiceStateChange) => {
      setState(change.state);
      setTranscript(change.transcript || '');
      setMatchedCommand(change.matchedCommand || null);
      setError(change.error || '');
    });

    return unsubscribe;
  }, []);

  const toggleListening = useCallback((continuous?: boolean) => {
    const service = getVoiceCommandService();
    const prefs = loadPreferences();
    // Apply wake word preference each time
    service.setWakeWordEnabled(prefs.voiceWakeWordEnabled);
    // Use preference for continuous mode if not explicitly specified
    const useContinuous = continuous ?? prefs.voiceContinuousMode;
    return service.toggleListening(useContinuous);
  }, []);

  const startListening = useCallback((continuous?: boolean) => {
    const service = getVoiceCommandService();
    const prefs = loadPreferences();
    service.setWakeWordEnabled(prefs.voiceWakeWordEnabled);
    const useContinuous = continuous ?? prefs.voiceContinuousMode;
    return service.startListening(useContinuous);
  }, []);

  const stopListening = useCallback(() => {
    const service = getVoiceCommandService();
    service.stopListening();
  }, []);

  return {
    state,
    transcript,
    matchedCommand,
    error,
    isSupported,
    isListening: state === 'listening' || state === 'processing',
    toggleListening,
    startListening,
    stopListening,
  };
}

/**
 * Hook to register all default voice commands
 * Should be called once at app root level
 */
export function useRegisterVoiceCommands() {
  const {
    project,
    setMode,
    undo,
    redo,
    canUndo,
    canRedo,
    setAIAssistantOpen,
    aiAssistantOpen,
    sidebarOpen,
    setSidebarOpen,
  } = useAppStore();

  useEffect(() => {
    const service = getVoiceCommandService();
    if (!service.isSupported()) return;

    const coordinator = getPlaybackCoordinator();

    // Build commands with access to current state
    const commands: VoiceCommand[] = [
      // === PLAYBACK COMMANDS ===
      {
        id: 'play',
        phrases: ['play', 'start', 'go', 'play music', 'start playing', 'resume'],
        category: 'playback',
        description: 'Start playback',
        action: async () => {
          if (!project) {
            showInfo('No project loaded');
            return;
          }
          await coordinator.play();
          showSuccess('Playing');
        },
      },
      {
        id: 'pause',
        phrases: ['pause', 'hold', 'wait', 'pause playback'],
        category: 'playback',
        description: 'Pause playback',
        action: () => {
          coordinator.pause();
          showInfo('Paused');
        },
      },
      {
        id: 'stop',
        phrases: ['stop', 'stop playing', 'halt', 'end'],
        category: 'playback',
        description: 'Stop playback and reset position',
        action: () => {
          coordinator.stop();
          showInfo('Stopped');
        },
      },
      {
        id: 'rewind',
        phrases: ['rewind', 'go back', 'back to start', 'from the top', 'beginning', 'restart'],
        category: 'playback',
        description: 'Go back to the beginning',
        action: () => {
          coordinator.stop();
          showInfo('Rewound to start');
        },
      },

      // === RECORDING COMMANDS ===
      {
        id: 'record',
        phrases: ['record', 'start recording', 'begin recording', 'arm and record', 'rec'],
        category: 'recording',
        description: 'Start recording',
        action: async () => {
          if (!project) {
            showInfo('No project loaded');
            return;
          }
          // TODO: Implement actual recording
          showInfo('Recording started (demo)');
        },
      },
      {
        id: 'stop-recording',
        phrases: ['stop recording', 'end recording', 'finish recording', 'done recording'],
        category: 'recording',
        description: 'Stop recording',
        action: () => {
          showInfo('Recording stopped (demo)');
        },
      },

      // === NAVIGATION COMMANDS ===
      {
        id: 'go-compose',
        phrases: [
          'compose', 'go to compose', 'compose mode', 'switch to compose',
          'open compose', 'composition', 'write music', 'songwriting'
        ],
        category: 'navigation',
        description: 'Switch to Compose mode',
        action: () => {
          setMode('compose');
          showSuccess('Compose mode');
        },
      },
      {
        id: 'go-record',
        phrases: [
          'record mode', 'go to record', 'switch to record', 'open record',
          'recording studio', 'studio'
        ],
        category: 'navigation',
        description: 'Switch to Record mode',
        action: () => {
          setMode('record');
          showSuccess('Record mode');
        },
      },
      {
        id: 'go-mix',
        phrases: [
          'mix', 'go to mix', 'mix mode', 'switch to mix', 'open mix',
          'mixer', 'mixing', 'mixing console'
        ],
        category: 'navigation',
        description: 'Switch to Mix mode',
        action: () => {
          if (!project) {
            showInfo('Load a project first');
            return;
          }
          setMode('mix');
          showSuccess('Mix mode');
        },
      },
      {
        id: 'go-master',
        phrases: [
          'master', 'go to master', 'master mode', 'switch to master',
          'mastering', 'final mix'
        ],
        category: 'navigation',
        description: 'Switch to Master mode',
        action: () => {
          if (!project) {
            showInfo('Load a project first');
            return;
          }
          setMode('master');
          showSuccess('Master mode');
        },
      },
      {
        id: 'go-practice',
        phrases: [
          'practice', 'go to practice', 'practice mode', 'switch to practice',
          'training', 'learn', 'learning mode'
        ],
        category: 'navigation',
        description: 'Switch to Practice mode',
        action: () => {
          setMode('practice');
          showSuccess('Practice mode');
        },
      },
      {
        id: 'go-distribute',
        phrases: [
          'distribute', 'go to distribute', 'distribute mode', 'switch to distribute',
          'distribution', 'publish', 'release'
        ],
        category: 'navigation',
        description: 'Switch to Distribute mode',
        action: () => {
          if (!project) {
            showInfo('Load a project first');
            return;
          }
          setMode('distribute');
          showSuccess('Distribute mode');
        },
      },

      // === PROJECT COMMANDS ===
      {
        id: 'save',
        phrases: ['save', 'save project', 'save file', 'save my work', 'quick save'],
        category: 'project',
        description: 'Save the current project',
        action: () => {
          if (!project) {
            showInfo('No project to save');
            return;
          }
          saveProject(project);
          showSuccess('Project saved');
        },
      },
      {
        id: 'undo',
        phrases: ['undo', 'undo that', 'go back', 'take that back', 'oops'],
        category: 'project',
        description: 'Undo last action',
        action: () => {
          if (canUndo()) {
            undo();
            showInfo('Undo');
          } else {
            showInfo('Nothing to undo');
          }
        },
      },
      {
        id: 'redo',
        phrases: ['redo', 'redo that', 'bring it back', 'restore'],
        category: 'project',
        description: 'Redo last undone action',
        action: () => {
          if (canRedo()) {
            redo();
            showInfo('Redo');
          } else {
            showInfo('Nothing to redo');
          }
        },
      },

      // === SYSTEM COMMANDS ===
      {
        id: 'toggle-ai',
        phrases: [
          'ai', 'assistant', 'open ai', 'close ai', 'toggle ai',
          'ai assistant', 'show assistant', 'hide assistant', 'help me'
        ],
        category: 'system',
        description: 'Toggle AI Assistant panel',
        action: () => {
          setAIAssistantOpen(!aiAssistantOpen);
          showInfo(aiAssistantOpen ? 'AI hidden' : 'AI opened');
        },
      },
      {
        id: 'toggle-sidebar',
        phrases: [
          'sidebar', 'toggle sidebar', 'show sidebar', 'hide sidebar',
          'open sidebar', 'close sidebar', 'tracks panel'
        ],
        category: 'system',
        description: 'Toggle sidebar',
        action: () => {
          setSidebarOpen(!sidebarOpen);
          showInfo(sidebarOpen ? 'Sidebar hidden' : 'Sidebar opened');
        },
      },
      {
        id: 'stop-listening',
        phrases: [
          'stop listening', 'disable voice', 'voice off', 'stop voice',
          'be quiet', 'silence', "that's all"
        ],
        category: 'system',
        description: 'Stop voice recognition',
        action: () => {
          const service = getVoiceCommandService();
          service.stopListening();
          showInfo('Voice commands off');
        },
      },
      {
        id: 'help-voice',
        phrases: ['what can you do', 'help', 'voice help', 'list commands', 'commands'],
        category: 'system',
        description: 'Show available voice commands',
        action: () => {
          showInfo('Say: play, pause, stop, save, undo, compose, mix, master...');
        },
      },

      // === MIXING COMMANDS ===
      {
        id: 'mute-all',
        phrases: ['mute all', 'mute everything', 'silence all', 'all mute'],
        category: 'mixing',
        description: 'Mute all tracks',
        action: () => {
          showInfo('Muted all tracks (demo)');
        },
      },
      {
        id: 'unmute-all',
        phrases: ['unmute all', 'unmute everything', 'all unmute', 'bring back all'],
        category: 'mixing',
        description: 'Unmute all tracks',
        action: () => {
          showInfo('Unmuted all tracks (demo)');
        },
      },
      {
        id: 'solo-drums',
        phrases: ['solo drums', 'only drums', 'just drums', 'drums only'],
        category: 'mixing',
        description: 'Solo the drum track',
        action: () => {
          showInfo('Drums soloed (demo)');
        },
      },
      {
        id: 'solo-guitar',
        phrases: ['solo guitar', 'only guitar', 'just guitar', 'guitar only'],
        category: 'mixing',
        description: 'Solo the guitar track',
        action: () => {
          showInfo('Guitar soloed (demo)');
        },
      },
      {
        id: 'solo-bass',
        phrases: ['solo bass', 'only bass', 'just bass', 'bass only'],
        category: 'mixing',
        description: 'Solo the bass track',
        action: () => {
          showInfo('Bass soloed (demo)');
        },
      },
    ];

    // Register all commands
    service.registerCommands(commands);

    // Register enhanced commands (tempo, count-in, context-aware, etc.)
    const enhancedCommands = getEnhancedCommands();
    service.registerCommands(enhancedCommands);

    // Register music search commands (find backing tracks, jam along)
    const musicCommands = getMusicVoiceCommands();
    service.registerCommands(musicCommands);

    // Configure audio feedback from preferences
    const audioFeedback = getAudioFeedback();
    const prefs = loadPreferences();
    audioFeedback.setEnabled(prefs.voiceEnabled && prefs.voiceAudioFeedback);

    // Cleanup: unregister on unmount
    return () => {
      commands.forEach(cmd => service.unregisterCommand(cmd.id));
      enhancedCommands.forEach(cmd => service.unregisterCommand(cmd.id));
      musicCommands.forEach(cmd => service.unregisterCommand(cmd.id));
    };
  }, [project, setMode, undo, redo, canUndo, canRedo, setAIAssistantOpen, aiAssistantOpen, sidebarOpen, setSidebarOpen]);
}
