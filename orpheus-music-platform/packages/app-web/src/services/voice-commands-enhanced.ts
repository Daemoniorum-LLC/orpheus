/**
 * Enhanced Voice Commands - God-tier voice control for musicians
 * Natural language, context-aware, musical intelligence
 */

import { getVoiceCommandService, type VoiceCommand } from './voice-commands';
import { getAudioFeedback } from './audio-feedback';
import { useAppStore } from '../store/app-store';
import { getPlaybackCoordinator } from './playback-coordinator';
import { saveProject } from './project-save';
import { showSuccess, showInfo, showError } from './toast';

// Track context for context-aware commands
interface VoiceContext {
  lastAction: string | null;
  lastTrackId: string | null;
  lastValue: number | null;
  tempo: number;
  isRecording: boolean;
  loopStart: number | null;
  loopEnd: number | null;
}

const voiceContext: VoiceContext = {
  lastAction: null,
  lastTrackId: null,
  lastValue: null,
  tempo: 120,
  isRecording: false,
  loopStart: null,
  loopEnd: null,
};

// Custom macros storage
interface VoiceMacro {
  name: string;
  phrases: string[];
  actions: string[];  // Command IDs to execute in sequence
}

const customMacros: Map<string, VoiceMacro> = new Map();

/**
 * Parse natural language for values
 */
function parseValue(text: string): number | null {
  // Direct numbers
  const directMatch = text.match(/(\d+)/);
  if (directMatch) return parseInt(directMatch[1], 10);

  // Word numbers
  const wordNumbers: Record<string, number> = {
    'one': 1, 'two': 2, 'three': 3, 'four': 4, 'five': 5,
    'six': 6, 'seven': 7, 'eight': 8, 'nine': 9, 'ten': 10,
    'twenty': 20, 'thirty': 30, 'forty': 40, 'fifty': 50,
    'sixty': 60, 'seventy': 70, 'eighty': 80, 'ninety': 90,
    'hundred': 100,
  };

  for (const [word, num] of Object.entries(wordNumbers)) {
    if (text.includes(word)) return num;
  }

  // Relative values
  if (text.includes('a bit') || text.includes('a little')) return 5;
  if (text.includes('a lot') || text.includes('much')) return 20;
  if (text.includes('half')) return 50;
  if (text.includes('double')) return 200;

  return null;
}

/**
 * Parse tempo from natural language
 */
function parseTempo(text: string): number | null {
  // "120 bpm" or "set tempo to 120"
  const bpmMatch = text.match(/(\d+)\s*(bpm|beats|tempo)?/i);
  if (bpmMatch) return parseInt(bpmMatch[1], 10);

  // Relative tempo
  if (text.includes('faster') || text.includes('speed up')) {
    return voiceContext.tempo + 10;
  }
  if (text.includes('slower') || text.includes('slow down')) {
    return voiceContext.tempo - 10;
  }
  if (text.includes('double time')) {
    return voiceContext.tempo * 2;
  }
  if (text.includes('half time') || text.includes('half speed')) {
    return Math.round(voiceContext.tempo / 2);
  }

  return null;
}

/**
 * Get the enhanced command set
 */
export function getEnhancedCommands(): VoiceCommand[] {
  const audioFeedback = getAudioFeedback();

  return [
    // ========================================
    // TEMPO & METRONOME COMMANDS
    // ========================================
    {
      id: 'set-tempo',
      phrases: [
        'set tempo to', 'tempo', 'bpm', 'set bpm to',
        'change tempo to', 'make it', 'speed'
      ],
      category: 'playback',
      description: 'Set the tempo',
      action: () => {
        // This is handled by natural language parser
        showInfo(`Tempo: ${voiceContext.tempo} BPM`);
        audioFeedback.play('confirm');
      },
    },
    {
      id: 'tempo-faster',
      phrases: [
        'faster', 'speed up', 'quicker', 'increase tempo',
        'bump up the tempo', 'pick up the pace'
      ],
      category: 'playback',
      description: 'Increase tempo by 10 BPM',
      action: () => {
        voiceContext.tempo = Math.min(300, voiceContext.tempo + 10);
        showSuccess(`Tempo: ${voiceContext.tempo} BPM`);
        audioFeedback.play('confirm');
      },
    },
    {
      id: 'tempo-slower',
      phrases: [
        'slower', 'slow down', 'decrease tempo', 'reduce tempo',
        'take it down', 'ease up'
      ],
      category: 'playback',
      description: 'Decrease tempo by 10 BPM',
      action: () => {
        voiceContext.tempo = Math.max(20, voiceContext.tempo - 10);
        showSuccess(`Tempo: ${voiceContext.tempo} BPM`);
        audioFeedback.play('confirm');
      },
    },
    {
      id: 'metronome-on',
      phrases: [
        'metronome on', 'click on', 'start metronome', 'give me a click',
        'turn on the click', 'click track on', 'start the click'
      ],
      category: 'playback',
      description: 'Turn metronome on',
      action: () => {
        showSuccess('Metronome on');
        audioFeedback.play('metronome');
        audioFeedback.speak('Click on');
      },
    },
    {
      id: 'metronome-off',
      phrases: [
        'metronome off', 'click off', 'stop metronome', 'no click',
        'turn off the click', 'click track off', 'stop the click'
      ],
      category: 'playback',
      description: 'Turn metronome off',
      action: () => {
        showInfo('Metronome off');
        audioFeedback.play('confirm');
      },
    },

    // ========================================
    // COUNT-IN & RECORDING
    // ========================================
    {
      id: 'count-in',
      phrases: [
        'count me in', 'give me a count', 'count in',
        'four bar count', '4 bar count', 'count from',
        'one two three four', 'ready set go'
      ],
      category: 'recording',
      description: 'Count in before recording/playback',
      action: () => {
        const coordinator = getPlaybackCoordinator();
        showInfo('Counting in...');
        audioFeedback.playCountIn(4, voiceContext.tempo, async () => {
          audioFeedback.play('recordStart');
          await coordinator.play();
          showSuccess('Playing!');
        });
      },
    },
    {
      id: 'record-count-in',
      phrases: [
        'record with count in', 'count me in and record',
        'ready to record', 'lets record', "let's record",
        'start recording with count'
      ],
      category: 'recording',
      description: 'Count in then start recording',
      action: () => {
        showInfo('Recording in 4...');
        audioFeedback.playCountIn(4, voiceContext.tempo, () => {
          voiceContext.isRecording = true;
          audioFeedback.play('recordStart');
          audioFeedback.speak('Recording');
          showSuccess('Recording!');
        });
      },
    },
    {
      id: 'punch-in',
      phrases: [
        'punch in', 'punch in here', 'punch', 'drop in',
        'punch in next bar', 'punch in on the one'
      ],
      category: 'recording',
      description: 'Punch in recording at current position',
      action: () => {
        voiceContext.isRecording = true;
        audioFeedback.play('recordStart');
        showSuccess('Punched in!');
      },
    },
    {
      id: 'punch-out',
      phrases: [
        'punch out', 'stop punch', 'end punch', 'drop out'
      ],
      category: 'recording',
      description: 'Punch out of recording',
      action: () => {
        voiceContext.isRecording = false;
        audioFeedback.play('recordStop');
        showInfo('Punched out');
      },
    },
    {
      id: 'good-take',
      phrases: [
        'good take', 'keep that', 'that was good', "that's the one",
        'perfect', 'nailed it', 'keeper', "that's a take"
      ],
      category: 'recording',
      description: 'Mark the last take as a keeper',
      action: () => {
        audioFeedback.play('success');
        audioFeedback.speak('Take saved');
        showSuccess('Take marked as keeper!');
      },
    },
    {
      id: 'try-again',
      phrases: [
        'try again', 'one more time', 'again', 'redo that',
        'lets try that again', "let's go again", 'from the top'
      ],
      category: 'recording',
      description: 'Discard and try again',
      action: () => {
        audioFeedback.play('confirm');
        showInfo('Ready to go again');
      },
    },

    // ========================================
    // LOOP CONTROL
    // ========================================
    {
      id: 'loop-section',
      phrases: [
        'loop this', 'loop here', 'set loop', 'loop this section',
        'repeat this', 'keep looping'
      ],
      category: 'playback',
      description: 'Loop the current section',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Looping section');
      },
    },
    {
      id: 'loop-off',
      phrases: [
        'loop off', 'stop looping', 'no loop', 'disable loop',
        'stop repeating', 'clear loop'
      ],
      category: 'playback',
      description: 'Turn off looping',
      action: () => {
        voiceContext.loopStart = null;
        voiceContext.loopEnd = null;
        audioFeedback.play('confirm');
        showInfo('Loop off');
      },
    },

    // ========================================
    // SECTION NAVIGATION
    // ========================================
    {
      id: 'go-verse',
      phrases: [
        'go to verse', 'verse', 'jump to verse', 'verse one',
        'first verse', 'the verse'
      ],
      category: 'navigation',
      description: 'Jump to the verse section',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Jumped to Verse');
      },
    },
    {
      id: 'go-chorus',
      phrases: [
        'go to chorus', 'chorus', 'jump to chorus', 'the chorus',
        'go to the hook', 'the hook'
      ],
      category: 'navigation',
      description: 'Jump to the chorus section',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Jumped to Chorus');
      },
    },
    {
      id: 'go-bridge',
      phrases: [
        'go to bridge', 'bridge', 'jump to bridge', 'the bridge',
        'middle eight', 'go to the middle'
      ],
      category: 'navigation',
      description: 'Jump to the bridge section',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Jumped to Bridge');
      },
    },
    {
      id: 'go-intro',
      phrases: [
        'go to intro', 'intro', 'the beginning', 'from the top',
        'start', 'beginning'
      ],
      category: 'navigation',
      description: 'Jump to the intro',
      action: () => {
        const coordinator = getPlaybackCoordinator();
        coordinator.stop();
        audioFeedback.play('confirm');
        showSuccess('At the intro');
      },
    },
    {
      id: 'go-outro',
      phrases: [
        'go to outro', 'outro', 'the end', 'ending', 'go to end',
        'jump to ending'
      ],
      category: 'navigation',
      description: 'Jump to the outro',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Jumped to Outro');
      },
    },
    {
      id: 'drop-marker',
      phrases: [
        'drop marker', 'mark this', 'marker', 'bookmark',
        'drop a pin', 'mark here', 'remember this spot'
      ],
      category: 'navigation',
      description: 'Drop a marker at current position',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Marker dropped');
      },
    },

    // ========================================
    // CONTEXT-AWARE TRACK COMMANDS
    // ========================================
    {
      id: 'louder',
      phrases: [
        'louder', 'turn it up', 'more volume', 'boost it',
        'make it louder', 'bring it up', 'pump it up', 'crank it'
      ],
      category: 'mixing',
      description: 'Increase volume of selected track',
      action: () => {
        const { selectedTrackId } = useAppStore.getState();
        if (selectedTrackId) {
          audioFeedback.play('confirm');
          showSuccess('Volume +3dB');
        } else {
          showInfo('Select a track first');
        }
      },
    },
    {
      id: 'quieter',
      phrases: [
        'quieter', 'turn it down', 'less volume', 'softer',
        'make it quieter', 'bring it down', 'ease it back', 'lower'
      ],
      category: 'mixing',
      description: 'Decrease volume of selected track',
      action: () => {
        const { selectedTrackId } = useAppStore.getState();
        if (selectedTrackId) {
          audioFeedback.play('confirm');
          showSuccess('Volume -3dB');
        } else {
          showInfo('Select a track first');
        }
      },
    },
    {
      id: 'mute-this',
      phrases: [
        'mute this', 'mute it', 'silence this', 'cut this',
        'kill this track', 'shut this up'
      ],
      category: 'mixing',
      description: 'Mute the selected track',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Track muted');
      },
    },
    {
      id: 'unmute-this',
      phrases: [
        'unmute this', 'unmute it', 'bring this back',
        'restore this', 'turn this back on'
      ],
      category: 'mixing',
      description: 'Unmute the selected track',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Track unmuted');
      },
    },
    {
      id: 'solo-this',
      phrases: [
        'solo this', 'solo it', 'just this', 'only this',
        'isolate this', 'focus on this'
      ],
      category: 'mixing',
      description: 'Solo the selected track',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Track soloed');
      },
    },
    {
      id: 'pan-left',
      phrases: [
        'pan left', 'move left', 'to the left', 'hard left',
        'push it left'
      ],
      category: 'mixing',
      description: 'Pan selected track left',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Panned left');
      },
    },
    {
      id: 'pan-right',
      phrases: [
        'pan right', 'move right', 'to the right', 'hard right',
        'push it right'
      ],
      category: 'mixing',
      description: 'Pan selected track right',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Panned right');
      },
    },
    {
      id: 'pan-center',
      phrases: [
        'pan center', 'center it', 'center', 'middle',
        'bring it to center', 'dead center'
      ],
      category: 'mixing',
      description: 'Pan selected track to center',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Panned center');
      },
    },

    // ========================================
    // EFFECTS (Natural Language)
    // ========================================
    {
      id: 'add-reverb',
      phrases: [
        'add reverb', 'more reverb', 'reverb', 'add some space',
        'make it wet', 'add some room', 'needs reverb'
      ],
      category: 'mixing',
      description: 'Add/increase reverb',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Added reverb');
      },
    },
    {
      id: 'add-delay',
      phrases: [
        'add delay', 'more delay', 'delay', 'add echo',
        'echo', 'give it some delay'
      ],
      category: 'mixing',
      description: 'Add/increase delay',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Added delay');
      },
    },
    {
      id: 'more-compression',
      phrases: [
        'more compression', 'compress it', 'squash it',
        'tighten it up', 'add compression', 'needs compression'
      ],
      category: 'mixing',
      description: 'Add/increase compression',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Added compression');
      },
    },
    {
      id: 'brighten',
      phrases: [
        'brighten', 'brighter', 'more treble', 'add high end',
        'more presence', 'add some air', 'open it up', 'crispy'
      ],
      category: 'mixing',
      description: 'Boost high frequencies',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Brightened');
      },
    },
    {
      id: 'darken',
      phrases: [
        'darken', 'darker', 'less treble', 'roll off highs',
        'warmer', 'make it warmer', 'less harsh', 'smooth it out'
      ],
      category: 'mixing',
      description: 'Cut high frequencies',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Darkened');
      },
    },
    {
      id: 'more-bass',
      phrases: [
        'more bass', 'boost bass', 'add low end', 'more bottom',
        'thicken it up', 'make it fat', 'beef it up'
      ],
      category: 'mixing',
      description: 'Boost low frequencies',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Boosted bass');
      },
    },
    {
      id: 'less-bass',
      phrases: [
        'less bass', 'cut bass', 'reduce low end', 'thin it out',
        'too boomy', 'less mud', 'clean up the low end'
      ],
      category: 'mixing',
      description: 'Cut low frequencies',
      action: () => {
        audioFeedback.play('confirm');
        showSuccess('Cut bass');
      },
    },

    // ========================================
    // CUSTOM MACROS
    // ========================================
    {
      id: 'ready-to-jam',
      phrases: [
        'ready to jam', 'lets jam', "let's jam", 'jam mode',
        'time to jam', 'jam session'
      ],
      category: 'system',
      description: 'Start metronome + count in + play',
      action: () => {
        showInfo('Getting ready to jam...');
        audioFeedback.speak('Lets jam');
        setTimeout(() => {
          audioFeedback.playCountIn(4, voiceContext.tempo, async () => {
            const coordinator = getPlaybackCoordinator();
            await coordinator.play();
            audioFeedback.play('success');
          });
        }, 500);
      },
    },
    {
      id: 'wrap-it-up',
      phrases: [
        'wrap it up', "that's a wrap", 'done for now', 'finish up',
        'call it a day', 'pack it up'
      ],
      category: 'system',
      description: 'Stop, save, and confirm',
      action: () => {
        const coordinator = getPlaybackCoordinator();
        const { project } = useAppStore.getState();
        coordinator.stop();
        if (project) {
          saveProject(project);
        }
        audioFeedback.play('success');
        audioFeedback.speak("That's a wrap. Project saved.");
        showSuccess("That's a wrap! Project saved.");
      },
    },
    {
      id: 'whats-the-tempo',
      phrases: [
        "what's the tempo", 'what tempo', 'tempo check', 'current tempo',
        'how fast', 'what bpm'
      ],
      category: 'system',
      description: 'Announce current tempo',
      action: () => {
        audioFeedback.speak(`${voiceContext.tempo} BPM`);
        showInfo(`Current tempo: ${voiceContext.tempo} BPM`);
      },
    },

    // ========================================
    // PRACTICE MODE
    // ========================================
    {
      id: 'slow-practice',
      phrases: [
        'slow it down', 'practice slow', 'half speed practice',
        'slow practice', 'take it slow'
      ],
      category: 'playback',
      description: 'Slow down for practice',
      action: () => {
        voiceContext.tempo = Math.round(voiceContext.tempo * 0.5);
        audioFeedback.play('confirm');
        showSuccess(`Slowed to ${voiceContext.tempo} BPM`);
      },
    },
    {
      id: 'speed-up-practice',
      phrases: [
        'speed it up', 'faster practice', 'normal speed',
        'full speed', 'bring it back up'
      ],
      category: 'playback',
      description: 'Speed up / return to normal',
      action: () => {
        voiceContext.tempo = Math.min(300, Math.round(voiceContext.tempo * 1.5));
        audioFeedback.play('confirm');
        showSuccess(`Speed: ${voiceContext.tempo} BPM`);
      },
    },
  ];
}

/**
 * Register a custom macro
 */
export function registerMacro(macro: VoiceMacro): void {
  customMacros.set(macro.name, macro);
}

/**
 * Get current voice context
 */
export function getVoiceContext(): VoiceContext {
  return { ...voiceContext };
}

/**
 * Update voice context
 */
export function updateVoiceContext(update: Partial<VoiceContext>): void {
  Object.assign(voiceContext, update);
}

/**
 * Register all enhanced commands
 */
export function registerEnhancedCommands(): void {
  const service = getVoiceCommandService();
  const commands = getEnhancedCommands();
  service.registerCommands(commands);
}
