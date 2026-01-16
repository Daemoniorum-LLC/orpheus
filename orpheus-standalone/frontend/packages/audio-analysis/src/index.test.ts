import { describe, it, expect } from 'vitest';
import * as AudioAnalysis from './index';

describe('index.ts - Package exports', () => {
  describe('peak-detector exports', () => {
    it('should export PeakDetector class', () => {
      expect(AudioAnalysis.PeakDetector).toBeDefined();
      expect(typeof AudioAnalysis.PeakDetector).toBe('function');
    });

    it('should export TruePeakDetector class', () => {
      expect(AudioAnalysis.TruePeakDetector).toBeDefined();
      expect(typeof AudioAnalysis.TruePeakDetector).toBe('function');
    });

    it('should be able to create PeakDetector instance', () => {
      const detector = new AudioAnalysis.PeakDetector();
      expect(detector).toBeInstanceOf(AudioAnalysis.PeakDetector);
    });

    it('should be able to create TruePeakDetector instance', () => {
      const detector = new AudioAnalysis.TruePeakDetector();
      expect(detector).toBeInstanceOf(AudioAnalysis.TruePeakDetector);
    });
  });

  describe('rms-meter exports', () => {
    it('should export RMSMeter class', () => {
      expect(AudioAnalysis.RMSMeter).toBeDefined();
      expect(typeof AudioAnalysis.RMSMeter).toBe('function');
    });

    it('should export VUMeter class', () => {
      expect(AudioAnalysis.VUMeter).toBeDefined();
      expect(typeof AudioAnalysis.VUMeter).toBe('function');
    });

    it('should be able to create RMSMeter instance', () => {
      const meter = new AudioAnalysis.RMSMeter();
      expect(meter).toBeInstanceOf(AudioAnalysis.RMSMeter);
    });

    it('should be able to create VUMeter instance', () => {
      const meter = new AudioAnalysis.VUMeter();
      expect(meter).toBeInstanceOf(AudioAnalysis.VUMeter);
    });
  });

  describe('lufs-meter exports', () => {
    it('should export LUFSMeter class', () => {
      expect(AudioAnalysis.LUFSMeter).toBeDefined();
      expect(typeof AudioAnalysis.LUFSMeter).toBe('function');
    });

    it('should export normalizeLUFS function', () => {
      expect(AudioAnalysis.normalizeLUFS).toBeDefined();
      expect(typeof AudioAnalysis.normalizeLUFS).toBe('function');
    });

    it('should export LOUDNESS_TARGETS constant', () => {
      expect(AudioAnalysis.LOUDNESS_TARGETS).toBeDefined();
      expect(typeof AudioAnalysis.LOUDNESS_TARGETS).toBe('object');
    });

    it('should be able to create LUFSMeter instance', () => {
      const meter = new AudioAnalysis.LUFSMeter();
      expect(meter).toBeInstanceOf(AudioAnalysis.LUFSMeter);
    });

    it('should have all loudness targets defined', () => {
      expect(AudioAnalysis.LOUDNESS_TARGETS.SPOTIFY).toBe(-14);
      expect(AudioAnalysis.LOUDNESS_TARGETS.APPLE_MUSIC).toBe(-16);
      expect(AudioAnalysis.LOUDNESS_TARGETS.YOUTUBE).toBe(-14);
      expect(AudioAnalysis.LOUDNESS_TARGETS.BROADCAST_TV).toBe(-23);
    });
  });

  describe('frequency-analyzer exports', () => {
    it('should export FrequencyAnalyzer class', () => {
      expect(AudioAnalysis.FrequencyAnalyzer).toBeDefined();
      expect(typeof AudioAnalysis.FrequencyAnalyzer).toBe('function');
    });

    it('should export FREQUENCY_BANDS constant', () => {
      expect(AudioAnalysis.FREQUENCY_BANDS).toBeDefined();
      expect(typeof AudioAnalysis.FREQUENCY_BANDS).toBe('object');
    });

    it('should export PROBLEMATIC_FREQUENCIES constant', () => {
      expect(AudioAnalysis.PROBLEMATIC_FREQUENCIES).toBeDefined();
      expect(typeof AudioAnalysis.PROBLEMATIC_FREQUENCIES).toBe('object');
    });

    it('should be able to create FrequencyAnalyzer instance', () => {
      const analyzer = new AudioAnalysis.FrequencyAnalyzer();
      expect(analyzer).toBeInstanceOf(AudioAnalysis.FrequencyAnalyzer);
    });

    it('should have all frequency bands defined', () => {
      expect(AudioAnalysis.FREQUENCY_BANDS.SUB_BASS).toBeDefined();
      expect(AudioAnalysis.FREQUENCY_BANDS.BASS).toBeDefined();
      expect(AudioAnalysis.FREQUENCY_BANDS.LOW_MIDS).toBeDefined();
      expect(AudioAnalysis.FREQUENCY_BANDS.MIDS).toBeDefined();
      expect(AudioAnalysis.FREQUENCY_BANDS.HIGH_MIDS).toBeDefined();
      expect(AudioAnalysis.FREQUENCY_BANDS.PRESENCE).toBeDefined();
      expect(AudioAnalysis.FREQUENCY_BANDS.BRILLIANCE).toBeDefined();
    });

    it('should have all problematic frequencies defined', () => {
      expect(AudioAnalysis.PROBLEMATIC_FREQUENCIES.MUD).toBeDefined();
      expect(AudioAnalysis.PROBLEMATIC_FREQUENCIES.BOXINESS).toBeDefined();
      expect(AudioAnalysis.PROBLEMATIC_FREQUENCIES.HARSHNESS).toBeDefined();
      expect(AudioAnalysis.PROBLEMATIC_FREQUENCIES.SIBILANCE).toBeDefined();
    });
  });

  describe('phase-meter exports', () => {
    it('should export PhaseMeter class', () => {
      expect(AudioAnalysis.PhaseMeter).toBeDefined();
      expect(typeof AudioAnalysis.PhaseMeter).toBe('function');
    });

    it('should export StereoWidthAnalyzer class', () => {
      expect(AudioAnalysis.StereoWidthAnalyzer).toBeDefined();
      expect(typeof AudioAnalysis.StereoWidthAnalyzer).toBe('function');
    });

    it('should export Goniometer class', () => {
      expect(AudioAnalysis.Goniometer).toBeDefined();
      expect(typeof AudioAnalysis.Goniometer).toBe('function');
    });

    it('should be able to create PhaseMeter instance', () => {
      const meter = new AudioAnalysis.PhaseMeter();
      expect(meter).toBeInstanceOf(AudioAnalysis.PhaseMeter);
    });

    it('should be able to create StereoWidthAnalyzer instance', () => {
      const analyzer = new AudioAnalysis.StereoWidthAnalyzer();
      expect(analyzer).toBeInstanceOf(AudioAnalysis.StereoWidthAnalyzer);
    });

    it('should be able to create Goniometer instance', () => {
      const goniometer = new AudioAnalysis.Goniometer();
      expect(goniometer).toBeInstanceOf(AudioAnalysis.Goniometer);
    });
  });

  describe('integration - all modules work together', () => {
    it('should use PeakDetector and RMSMeter together', () => {
      const peakDetector = new AudioAnalysis.PeakDetector();
      const rmsMeter = new AudioAnalysis.RMSMeter();

      const samples = new Float32Array(4096);
      for (let i = 0; i < samples.length; i++) {
        samples[i] = 0.5 * Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const peakAnalysis = peakDetector.analyze(samples);
      const rmsAnalysis = rmsMeter.analyze(samples);

      // Peak should be higher than RMS
      expect(peakAnalysis.peakLinear).toBeGreaterThanOrEqual(rmsAnalysis.rmsLinear);
      expect(peakAnalysis.peakDb).toBeGreaterThanOrEqual(rmsAnalysis.rmsDb);
    });

    it('should use LUFSMeter and normalizeLUFS together', () => {
      const channels = [
        new Float32Array(88200).fill(0.1),
        new Float32Array(88200).fill(0.1),
      ];

      const targetLUFS = AudioAnalysis.LOUDNESS_TARGETS.SPOTIFY;
      const normalized = AudioAnalysis.normalizeLUFS(channels, targetLUFS);

      const meter = new AudioAnalysis.LUFSMeter({ targetLoudness: targetLUFS });
      const analysis = meter.analyze(normalized);

      // Should be close to target
      expect(Math.abs(analysis.integrated - targetLUFS)).toBeLessThan(2);
    });

    it('should use FrequencyAnalyzer and PhaseMeter together', () => {
      const left = new Float32Array(4096);
      const right = new Float32Array(4096);

      for (let i = 0; i < left.length; i++) {
        left[i] = 0.5 * Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = 0.5 * Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const freqAnalyzer = new AudioAnalysis.FrequencyAnalyzer({ fftSize: 4096 });
      const phaseMeter = new AudioAnalysis.PhaseMeter();

      const spectrum = freqAnalyzer.analyze(left);
      const phaseAnalysis = phaseMeter.analyze(left, right);

      // Both should complete successfully
      expect(spectrum.magnitudes.length).toBeGreaterThan(0);
      expect(phaseAnalysis.correlation).toBeCloseTo(1.0, 5);
    });

    it('should analyze complete audio signal with all tools', () => {
      // Create test signal
      const samples = new Float32Array(8192);
      const left = new Float32Array(8192);
      const right = new Float32Array(8192);

      for (let i = 0; i < samples.length; i++) {
        samples[i] = 0.3 * Math.sin(2 * Math.PI * 440 * i / 44100);
        left[i] = samples[i];
        right[i] = samples[i] * 0.8;
      }

      // Peak analysis
      const peakDetector = new AudioAnalysis.PeakDetector();
      const peakAnalysis = peakDetector.analyze(samples);

      // RMS analysis
      const rmsMeter = new AudioAnalysis.RMSMeter();
      const rmsAnalysis = rmsMeter.analyze(samples);

      // Frequency analysis
      const freqAnalyzer = new AudioAnalysis.FrequencyAnalyzer({ fftSize: 2048 });
      const spectrum = freqAnalyzer.analyze(samples);
      const bands = freqAnalyzer.analyzeBands(samples);
      const pitch = freqAnalyzer.detectPitch(samples);

      // LUFS analysis
      const lufsMeter = new AudioAnalysis.LUFSMeter();
      const lufsAnalysis = lufsMeter.analyze([left, right]);

      // Phase analysis
      const phaseMeter = new AudioAnalysis.PhaseMeter();
      const phaseAnalysis = phaseMeter.analyze(left, right);

      // Stereo width
      const widthAnalyzer = new AudioAnalysis.StereoWidthAnalyzer();
      const widthAnalysis = widthAnalyzer.analyze(left, right);

      // Goniometer
      const goniometer = new AudioAnalysis.Goniometer();
      const gonioPoints = goniometer.generate(left, right);
      const gonioPattern = goniometer.analyzePattern(left, right);

      // Verify all analyses completed
      expect(peakAnalysis.peakLinear).toBeGreaterThan(0);
      expect(rmsAnalysis.rmsLinear).toBeGreaterThan(0);
      expect(spectrum.magnitudes.length).toBeGreaterThan(0);
      expect(bands.length).toBe(7);
      expect(pitch).toBeCloseTo(440, 20);
      expect(isFinite(lufsAnalysis.integrated)).toBe(true);
      expect(phaseAnalysis.correlation).toBeGreaterThan(0.5);
      expect(widthAnalysis.width).toBeGreaterThanOrEqual(0);
      expect(gonioPoints.length).toBeGreaterThan(0);
      expect(gonioPattern.shape).toBeDefined();
    });

    it('should detect and analyze phase issues', () => {
      const left = new Float32Array(4096);
      const right = new Float32Array(4096);

      // Create out-of-phase signal
      for (let i = 0; i < left.length; i++) {
        left[i] = 0.5 * Math.sin(2 * Math.PI * 440 * i / 44100);
        right[i] = -0.5 * Math.sin(2 * Math.PI * 440 * i / 44100);
      }

      const phaseMeter = new AudioAnalysis.PhaseMeter();
      const widthAnalyzer = new AudioAnalysis.StereoWidthAnalyzer();
      const goniometer = new AudioAnalysis.Goniometer();

      const phaseAnalysis = phaseMeter.analyze(left, right);
      const isInverted = phaseMeter.detectPhaseInversion(left, right);
      const suggestion = phaseMeter.suggestCorrection(phaseAnalysis);
      const widthAnalysis = widthAnalyzer.analyze(left, right);
      const pattern = goniometer.analyzePattern(left, right);

      // All should detect phase issues
      expect(phaseAnalysis.correlation).toBeLessThan(0);
      expect(isInverted).toBe(true);
      expect(suggestion).toBeTruthy();
      expect(widthAnalysis.width).toBeGreaterThan(1); // Wide due to phase
      expect(pattern.shape).toBe('horizontal');
    });

    it('should analyze frequency bands and detect peaks', () => {
      const samples = new Float32Array(8192);

      // Create signal with multiple frequencies
      for (let i = 0; i < samples.length; i++) {
        samples[i] =
          0.3 * Math.sin(2 * Math.PI * 80 * i / 44100) + // Bass
          0.3 * Math.sin(2 * Math.PI * 1000 * i / 44100) + // Mids
          0.2 * Math.sin(2 * Math.PI * 8000 * i / 44100); // Highs
      }

      const freqAnalyzer = new AudioAnalysis.FrequencyAnalyzer({ fftSize: 8192 });
      const peakDetector = new AudioAnalysis.PeakDetector();

      const bands = freqAnalyzer.analyzeBands(samples);
      const peakAnalysis = peakDetector.analyze(samples);

      // Bass band should have energy
      const bassBand = bands.find(b => b.name === 'Bass');
      expect(bassBand!.magnitude).toBeGreaterThan(-40);

      // Mids band should have energy
      const midBand = bands.find(b => b.name === 'Mids');
      expect(midBand!.magnitude).toBeGreaterThan(-40);

      // Peak should be sum of components
      expect(peakAnalysis.peakLinear).toBeGreaterThan(0.5);
    });

    it('should measure loudness for different platforms', () => {
      const channels = [
        new Float32Array(88200).fill(0.15),
        new Float32Array(88200).fill(0.15),
      ];

      const platforms = [
        { name: 'Spotify', target: AudioAnalysis.LOUDNESS_TARGETS.SPOTIFY },
        { name: 'Apple Music', target: AudioAnalysis.LOUDNESS_TARGETS.APPLE_MUSIC },
        { name: 'YouTube', target: AudioAnalysis.LOUDNESS_TARGETS.YOUTUBE },
        { name: 'Broadcast TV', target: AudioAnalysis.LOUDNESS_TARGETS.BROADCAST_TV },
      ];

      platforms.forEach(platform => {
        const normalized = AudioAnalysis.normalizeLUFS(channels, platform.target);
        const meter = new AudioAnalysis.LUFSMeter({ targetLoudness: platform.target });
        const analysis = meter.analyze(normalized);

        expect(Math.abs(analysis.integrated - platform.target)).toBeLessThan(2);
      });
    });

    it('should provide comprehensive metering for mastering', () => {
      const left = new Float32Array(88200); // 2 seconds
      const right = new Float32Array(88200);

      // Create realistic music signal
      for (let i = 0; i < left.length; i++) {
        const bass = 0.3 * Math.sin(2 * Math.PI * 80 * i / 44100);
        const mid = 0.2 * Math.sin(2 * Math.PI * 1000 * i / 44100);
        const high = 0.1 * Math.sin(2 * Math.PI * 8000 * i / 44100);

        left[i] = bass + mid + high;
        right[i] = bass + mid * 0.8 + high * 0.6;
      }

      // Complete mastering analysis
      const peakDetector = new AudioAnalysis.PeakDetector();
      const truePeakDetector = new AudioAnalysis.TruePeakDetector();
      const rmsMeter = new AudioAnalysis.RMSMeter();
      const lufsMeter = new AudioAnalysis.LUFSMeter();
      const freqAnalyzer = new AudioAnalysis.FrequencyAnalyzer({ fftSize: 4096 });
      const phaseMeter = new AudioAnalysis.PhaseMeter();

      const peakStereo = peakDetector.analyzeStereo(left, right);
      const truePeakLeft = truePeakDetector.analyze(left);
      const rmsStereo = rmsMeter.analyzeStereo(left, right);
      const lufsAnalysis = lufsMeter.analyze([left, right]);
      const bands = freqAnalyzer.analyzeBands(left);
      const phaseAnalysis = phaseMeter.analyze(left, right);

      // Verify complete analysis
      expect(peakStereo.left.peakLinear).toBeGreaterThan(0);
      expect(peakStereo.right.peakLinear).toBeGreaterThan(0);
      expect(truePeakLeft.peakLinear).toBeGreaterThan(0);
      expect(rmsStereo.left.rmsLinear).toBeGreaterThan(0);
      expect(isFinite(lufsAnalysis.integrated)).toBe(true);
      expect(bands.length).toBe(7);
      expect(phaseAnalysis.status).toBeDefined();
    });
  });

  describe('module completeness', () => {
    it('should have complete peak detection functionality', () => {
      const peakDetector = new AudioAnalysis.PeakDetector();
      const samples = new Float32Array(1000).fill(0.5);

      const analysis = peakDetector.analyze(samples);

      expect(analysis).toHaveProperty('peakDb');
      expect(analysis).toHaveProperty('peakLinear');
      expect(analysis).toHaveProperty('peakSampleIndex');
      expect(analysis).toHaveProperty('clipped');
      expect(analysis).toHaveProperty('clippedSamples');
      expect(analysis).toHaveProperty('crestFactor');
    });

    it('should have complete RMS metering functionality', () => {
      const rmsMeter = new AudioAnalysis.RMSMeter();
      const samples = new Float32Array(1000).fill(0.5);

      const analysis = rmsMeter.analyze(samples);

      expect(analysis).toHaveProperty('rmsDb');
      expect(analysis).toHaveProperty('rmsLinear');
      expect(analysis).toHaveProperty('dynamicRange');
      expect(analysis).toHaveProperty('peakDb');
    });

    it('should have complete LUFS metering functionality', () => {
      const lufsMeter = new AudioAnalysis.LUFSMeter();
      const channels = [new Float32Array(88200).fill(0.1), new Float32Array(88200).fill(0.1)];

      const analysis = lufsMeter.analyze(channels);

      expect(analysis).toHaveProperty('integrated');
      expect(analysis).toHaveProperty('range');
      expect(analysis).toHaveProperty('shortTerm');
      expect(analysis).toHaveProperty('momentary');
      expect(analysis).toHaveProperty('truePeak');
      expect(analysis).toHaveProperty('compliant');
      expect(analysis).toHaveProperty('target');
    });

    it('should have complete frequency analysis functionality', () => {
      const freqAnalyzer = new AudioAnalysis.FrequencyAnalyzer({ fftSize: 2048 });
      const samples = new Float32Array(2048).fill(0.5);

      const spectrum = freqAnalyzer.analyze(samples);

      expect(spectrum).toHaveProperty('frequencies');
      expect(spectrum).toHaveProperty('magnitudes');
      expect(spectrum).toHaveProperty('fftSize');
      expect(spectrum).toHaveProperty('sampleRate');
      expect(spectrum).toHaveProperty('resolution');
    });

    it('should have complete phase analysis functionality', () => {
      const phaseMeter = new AudioAnalysis.PhaseMeter();
      const left = new Float32Array(1000).fill(0.5);
      const right = new Float32Array(1000).fill(0.5);

      const analysis = phaseMeter.analyze(left, right);

      expect(analysis).toHaveProperty('correlation');
      expect(analysis).toHaveProperty('leftLevel');
      expect(analysis).toHaveProperty('rightLevel');
      expect(analysis).toHaveProperty('midLevel');
      expect(analysis).toHaveProperty('sideLevel');
      expect(analysis).toHaveProperty('coherence');
      expect(analysis).toHaveProperty('monoCompatible');
      expect(analysis).toHaveProperty('status');
    });

    it('should have all standard frequency bands', () => {
      expect(Object.keys(AudioAnalysis.FREQUENCY_BANDS).length).toBe(7);
    });

    it('should have all problematic frequency ranges', () => {
      expect(Object.keys(AudioAnalysis.PROBLEMATIC_FREQUENCIES).length).toBe(4);
    });

    it('should have all loudness targets', () => {
      expect(Object.keys(AudioAnalysis.LOUDNESS_TARGETS).length).toBeGreaterThanOrEqual(12);
    });
  });

  describe('type exports', () => {
    it('should be able to use exported types', () => {
      // This test verifies that types are properly exported
      // by using them in variable declarations

      const peakAnalysis: AudioAnalysis.PeakAnalysis = {
        peakDb: 0,
        peakLinear: 0.5,
        peakSampleIndex: 100,
        clipped: false,
        clippedSamples: 0,
        crestFactor: 10,
      };

      const rmsAnalysis: AudioAnalysis.RMSAnalysis = {
        rmsDb: -6,
        rmsLinear: 0.5,
        dynamicRange: 10,
        peakDb: 0,
      };

      const lufsAnalysis: AudioAnalysis.LUFSAnalysis = {
        integrated: -14,
        range: 5,
        shortTerm: -14,
        momentary: -14,
        truePeak: -1,
        compliant: true,
        target: -14,
      };

      const phaseAnalysis: AudioAnalysis.PhaseAnalysis = {
        correlation: 1.0,
        leftLevel: -6,
        rightLevel: -6,
        midLevel: -6,
        sideLevel: -20,
        coherence: 0.95,
        monoCompatible: true,
        status: 'good',
      };

      expect(peakAnalysis.peakLinear).toBe(0.5);
      expect(rmsAnalysis.rmsDb).toBe(-6);
      expect(lufsAnalysis.integrated).toBe(-14);
      expect(phaseAnalysis.correlation).toBe(1.0);
    });
  });
});
