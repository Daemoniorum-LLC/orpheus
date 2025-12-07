/**
 * Audio Processing Service
 * Real-time audio processing with Tone.js effects chain
 */

import * as Tone from 'tone';
import type { EQBand } from '../components/ParametricEQ';
import type { CompressorSettings } from '../components/Compressor';
import type { EffectsSettings } from '../components/EffectsRack';
import type { MasteringChainSettings } from '../components/MasteringChain';

/**
 * Channel Audio Processor
 * Manages the effects chain for a single channel
 */
export class ChannelProcessor {
  private input: Tone.Gain;
  private output: Tone.Gain;

  // Effects chain
  private eq: Tone.EQ3;
  private compressor: Tone.Compressor;
  private reverb: Tone.Reverb;
  private delay: Tone.FeedbackDelay;
  private reverbWet: Tone.CrossFade;
  private delayWet: Tone.CrossFade;

  constructor() {
    // Create signal chain
    this.input = new Tone.Gain(1);
    this.output = new Tone.Gain(1);

    // Create effects
    this.eq = new Tone.EQ3();
    this.compressor = new Tone.Compressor();
    this.reverb = new Tone.Reverb({ decay: 1.5, wet: 0 });
    this.delay = new Tone.FeedbackDelay({ delayTime: 0.25, feedback: 0.4, wet: 0 });

    // Wet/dry controls
    this.reverbWet = new Tone.CrossFade(0);
    this.delayWet = new Tone.CrossFade(0);

    // Connect signal chain: Input → EQ → Compressor → Reverb (wet/dry) → Delay (wet/dry) → Output
    this.input.connect(this.eq);
    this.eq.connect(this.compressor);

    // Reverb wet/dry
    this.compressor.connect(this.reverbWet.a); // Dry signal
    this.compressor.connect(this.reverb); // Wet signal
    this.reverb.connect(this.reverbWet.b);

    // Delay wet/dry
    this.reverbWet.connect(this.delayWet.a); // Dry signal
    this.reverbWet.connect(this.delay); // Wet signal
    this.delay.connect(this.delayWet.b);

    this.delayWet.connect(this.output);

    console.log('[ChannelProcessor] Initialized audio chain');
  }

  /**
   * Update EQ settings
   */
  updateEQ(bands: EQBand[]): void {
    if (bands.length >= 4) {
      // Map 4 bands to EQ3 (low, mid, high)
      // Band 0 (100Hz) → low
      // Band 1-2 (400Hz, 2kHz) → mid (average)
      // Band 3 (8kHz) → high

      this.eq.low.value = bands[0].gain;
      this.eq.mid.value = (bands[1].gain + bands[2].gain) / 2;
      this.eq.high.value = bands[3].gain;
    }
  }

  /**
   * Update compressor settings
   */
  updateCompressor(settings: CompressorSettings): void {
    if (!settings.enabled) {
      // Bypass compressor
      this.compressor.threshold.value = 0;
      this.compressor.ratio.value = 1;
      return;
    }

    this.compressor.threshold.value = settings.threshold;
    this.compressor.ratio.value = settings.ratio;
    this.compressor.attack.value = settings.attack / 1000; // Convert ms to seconds
    this.compressor.release.value = settings.release / 1000;
    this.compressor.knee.value = settings.knee;
  }

  /**
   * Update effects settings
   */
  updateEffects(settings: EffectsSettings): void {
    // Reverb
    if (settings.reverb.enabled) {
      this.reverb.decay = settings.reverb.decay;
      this.reverb.preDelay = settings.reverb.preDelay / 1000; // Convert ms to seconds
      this.reverbWet.fade.value = settings.reverb.wetDry / 100;
    } else {
      this.reverbWet.fade.value = 0; // Fully dry
    }

    // Delay
    if (settings.delay.enabled) {
      this.delay.delayTime.value = settings.delay.time / 1000; // Convert ms to seconds
      this.delay.feedback.value = settings.delay.feedback / 100;
      this.delayWet.fade.value = settings.delay.wetDry / 100;
    } else {
      this.delayWet.fade.value = 0; // Fully dry
    }
  }

  /**
   * Connect input source
   */
  connect(destination: Tone.ToneAudioNode): this {
    this.output.connect(destination);
    return this;
  }

  /**
   * Get input node
   */
  getInput(): Tone.Gain {
    return this.input;
  }

  /**
   * Get output node
   */
  getOutput(): Tone.Gain {
    return this.output;
  }

  /**
   * Dispose all effects
   */
  dispose(): void {
    this.input.dispose();
    this.output.dispose();
    this.eq.dispose();
    this.compressor.dispose();
    this.reverb.dispose();
    this.delay.dispose();
    this.reverbWet.dispose();
    this.delayWet.dispose();
  }
}

/**
 * Master Bus Processor
 * Manages the mastering chain
 */
export class MasterProcessor {
  private input: Tone.Gain;
  private output: Tone.Gain;

  // Mastering chain
  private eq: Tone.EQ3;
  private compressor: Tone.Compressor;
  private stereoWidener: Tone.StereoWidener;
  private limiter: Tone.Limiter;

  constructor() {
    this.input = new Tone.Gain(1);
    this.output = new Tone.Gain(1);

    // Create mastering effects
    this.eq = new Tone.EQ3();
    this.compressor = new Tone.Compressor({
      threshold: -6,
      ratio: 1.5,
      attack: 0.03,
      release: 0.3,
    });
    this.stereoWidener = new Tone.StereoWidener(1); // 100% width
    this.limiter = new Tone.Limiter(-0.3); // -0.3 dBFS ceiling

    // Connect mastering chain: Input → EQ → Compressor → Stereo → Limiter → Output
    this.input.chain(
      this.eq,
      this.compressor,
      this.stereoWidener,
      this.limiter,
      this.output
    );

    console.log('[MasterProcessor] Initialized mastering chain');
  }

  /**
   * Update mastering chain settings
   */
  updateChain(settings: MasteringChainSettings): void {
    // EQ
    if (settings.eq.enabled) {
      this.eq.low.value = settings.eq.lowShelf;
      this.eq.mid.value = settings.eq.presence;
      this.eq.high.value = settings.eq.airBand;
      this.eq.lowFrequency.value = Math.max(settings.eq.lowCut, 20);
    } else {
      this.eq.low.value = 0;
      this.eq.mid.value = 0;
      this.eq.high.value = 0;
    }

    // Compressor
    if (settings.compressor.enabled) {
      this.compressor.threshold.value = settings.compressor.threshold;
      this.compressor.ratio.value = settings.compressor.ratio;
      this.compressor.attack.value = settings.compressor.attack / 1000;
      this.compressor.release.value = settings.compressor.release / 1000;
    } else {
      this.compressor.threshold.value = 0;
      this.compressor.ratio.value = 1;
    }

    // Stereo
    if (settings.stereo.enabled) {
      this.stereoWidener.width.value = settings.stereo.width / 100;
    } else {
      this.stereoWidener.width.value = 1;
    }

    // Limiter (threshold is read-only, set at initialization)
    // For dynamic ceiling changes, we'd need to recreate the limiter
    // For now, limiter is always enabled with -0.3dBFS ceiling
  }

  /**
   * Connect to destination
   */
  connect(destination: Tone.ToneAudioNode): this {
    this.output.connect(destination);
    return this;
  }

  /**
   * Get input node
   */
  getInput(): Tone.Gain {
    return this.input;
  }

  /**
   * Get output node
   */
  getOutput(): Tone.Gain {
    return this.output;
  }

  /**
   * Dispose all effects
   */
  dispose(): void {
    this.input.dispose();
    this.output.dispose();
    this.eq.dispose();
    this.compressor.dispose();
    this.stereoWidener.dispose();
    this.limiter.dispose();
  }
}

/**
 * Audio Processing Manager
 * Manages all channel processors and master bus
 */
export class AudioProcessingManager {
  private channels: Map<string, ChannelProcessor> = new Map();
  private master: MasterProcessor;

  constructor() {
    this.master = new MasterProcessor();
    this.master.connect(Tone.getDestination());
    console.log('[AudioProcessingManager] Initialized');
  }

  /**
   * Create or get a channel processor
   */
  getChannel(channelId: string): ChannelProcessor {
    if (!this.channels.has(channelId)) {
      const channel = new ChannelProcessor();
      channel.connect(this.master.getInput());
      this.channels.set(channelId, channel);
      console.log(`[AudioProcessingManager] Created channel: ${channelId}`);
    }
    return this.channels.get(channelId)!;
  }

  /**
   * Remove a channel processor
   */
  removeChannel(channelId: string): void {
    const channel = this.channels.get(channelId);
    if (channel) {
      channel.dispose();
      this.channels.delete(channelId);
      console.log(`[AudioProcessingManager] Removed channel: ${channelId}`);
    }
  }

  /**
   * Get master processor
   */
  getMaster(): MasterProcessor {
    return this.master;
  }

  /**
   * Dispose all processors
   */
  dispose(): void {
    this.channels.forEach((channel) => channel.dispose());
    this.channels.clear();
    this.master.dispose();
    console.log('[AudioProcessingManager] Disposed all processors');
  }
}

// Singleton instance
let processingManager: AudioProcessingManager | null = null;

/**
 * Get the global audio processing manager
 */
export function getAudioProcessingManager(): AudioProcessingManager {
  if (!processingManager) {
    processingManager = new AudioProcessingManager();
  }
  return processingManager;
}

/**
 * Reset the processing manager (for testing)
 */
export function resetAudioProcessingManager(): void {
  if (processingManager) {
    processingManager.dispose();
    processingManager = null;
  }
}
