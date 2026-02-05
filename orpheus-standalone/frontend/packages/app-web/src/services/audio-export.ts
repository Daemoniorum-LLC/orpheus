/**
 * Audio Export Service
 * Exports audio in various formats (WAV, MP3, FLAC)
 */

import * as Tone from 'tone';

export interface ExportOptions {
  format: 'wav' | 'mp3' | 'flac' | 'aac';
  sampleRate: 44100 | 48000;
  bitDepth: 16 | 24;
  channels: 1 | 2;
  normalize?: boolean;
  targetLUFS?: number;
}

export interface ExportResult {
  blob: Blob;
  filename: string;
  format: string;
  duration: number;
  fileSize: number;
}

/**
 * Audio Export Manager
 * Handles rendering and exporting audio to various formats
 */
export class AudioExporter {
  /**
   * Export audio buffer to WAV format
   */
  static async exportToWAV(
    audioBuffer: AudioBuffer,
    options: Partial<ExportOptions> = {}
  ): Promise<ExportResult> {
    const sampleRate = options.sampleRate || 44100;
    const bitDepth = options.bitDepth || 24;
    const channels = Math.min(audioBuffer.numberOfChannels, options.channels || 2);

    console.log(`[AudioExporter] Exporting to WAV: ${sampleRate}Hz, ${bitDepth}-bit, ${channels} channels`);

    // Create WAV file
    const wavBuffer = AudioExporter.encodeWAV(audioBuffer, sampleRate, bitDepth, channels);
    const blob = new Blob([wavBuffer], { type: 'audio/wav' });

    return {
      blob,
      filename: `export-${Date.now()}.wav`,
      format: `WAV ${sampleRate / 1000}kHz/${bitDepth}-bit`,
      duration: audioBuffer.duration,
      fileSize: blob.size,
    };
  }

  /**
   * Export audio buffer to MP3 format (falls back to WAV if encoder not available)
   */
  static async exportToMP3(
    audioBuffer: AudioBuffer,
    bitrate: number = 320
  ): Promise<ExportResult> {
    console.log(`[AudioExporter] Exporting to MP3: ${bitrate}kbps`);

    // For now, use WAV as fallback since lamejs is not bundled
    // In production, you would use lamejs or another MP3 encoder
    console.warn('[AudioExporter] MP3 encoder not available, falling back to WAV');

    const wavResult = await AudioExporter.exportToWAV(audioBuffer, {
      sampleRate: 44100,
      bitDepth: 16,
    });

    return {
      ...wavResult,
      filename: `export-${Date.now()}.wav`,
      format: `WAV (MP3 encoder unavailable)`,
    };
  }

  /**
   * Export audio buffer to FLAC format (falls back to WAV)
   */
  static async exportToFLAC(audioBuffer: AudioBuffer): Promise<ExportResult> {
    console.log('[AudioExporter] Exporting to FLAC');

    // FLAC encoding requires a separate encoder library
    // For now, fall back to high-quality WAV
    console.warn('[AudioExporter] FLAC encoder not available, falling back to WAV');

    const wavResult = await AudioExporter.exportToWAV(audioBuffer, {
      sampleRate: 48000,
      bitDepth: 24,
    });

    return {
      ...wavResult,
      filename: `export-${Date.now()}.wav`,
      format: `WAV 48kHz/24-bit (FLAC unavailable)`,
    };
  }

  /**
   * Export with format auto-detection
   */
  static async export(
    audioBuffer: AudioBuffer,
    options: ExportOptions
  ): Promise<ExportResult> {
    switch (options.format) {
      case 'mp3':
        return AudioExporter.exportToMP3(audioBuffer);
      case 'flac':
        return AudioExporter.exportToFLAC(audioBuffer);
      case 'aac':
        // AAC requires native encoder, fall back to WAV
        console.warn('[AudioExporter] AAC encoder not available, falling back to WAV');
        return AudioExporter.exportToWAV(audioBuffer, options);
      case 'wav':
      default:
        return AudioExporter.exportToWAV(audioBuffer, options);
    }
  }

  /**
   * Render audio from the current Tone.js context
   * Uses Tone.Offline to capture audio output
   */
  static async renderFromContext(
    duration: number,
    renderCallback: (transport: typeof Tone.Transport) => void
  ): Promise<AudioBuffer> {
    console.log(`[AudioExporter] Rendering ${duration}s of audio...`);

    const buffer = await Tone.Offline(async ({ transport }) => {
      renderCallback(transport);
      transport.start();
    }, duration);

    console.log(`[AudioExporter] Rendered ${buffer.duration}s audio buffer`);
    return buffer;
  }

  /**
   * Download an exported file
   */
  static downloadBlob(blob: Blob, filename: string): void {
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    console.log(`[AudioExporter] Downloaded: ${filename}`);
  }

  /**
   * Encode audio buffer to WAV format
   */
  private static encodeWAV(
    audioBuffer: AudioBuffer,
    targetSampleRate: number,
    bitDepth: number,
    channels: number
  ): ArrayBuffer {
    const numChannels = Math.min(audioBuffer.numberOfChannels, channels);
    const sampleRate = audioBuffer.sampleRate;
    const length = audioBuffer.length;

    // Get channel data
    const channelData: Float32Array[] = [];
    for (let i = 0; i < numChannels; i++) {
      channelData.push(audioBuffer.getChannelData(i));
    }

    // Interleave channels
    const interleaved = new Float32Array(length * numChannels);
    for (let i = 0; i < length; i++) {
      for (let ch = 0; ch < numChannels; ch++) {
        interleaved[i * numChannels + ch] = channelData[ch][i];
      }
    }

    // Calculate sizes
    const bytesPerSample = bitDepth / 8;
    const dataSize = interleaved.length * bytesPerSample;
    const headerSize = 44;
    const totalSize = headerSize + dataSize;

    // Create buffer
    const buffer = new ArrayBuffer(totalSize);
    const view = new DataView(buffer);

    // Write WAV header
    // "RIFF" chunk
    AudioExporter.writeString(view, 0, 'RIFF');
    view.setUint32(4, totalSize - 8, true); // File size - 8
    AudioExporter.writeString(view, 8, 'WAVE');

    // "fmt " sub-chunk
    AudioExporter.writeString(view, 12, 'fmt ');
    view.setUint32(16, 16, true); // Sub-chunk size (16 for PCM)
    view.setUint16(20, 1, true); // Audio format (1 = PCM)
    view.setUint16(22, numChannels, true);
    view.setUint32(24, sampleRate, true);
    view.setUint32(28, sampleRate * numChannels * bytesPerSample, true); // Byte rate
    view.setUint16(32, numChannels * bytesPerSample, true); // Block align
    view.setUint16(34, bitDepth, true);

    // "data" sub-chunk
    AudioExporter.writeString(view, 36, 'data');
    view.setUint32(40, dataSize, true);

    // Write audio data
    let offset = 44;
    if (bitDepth === 16) {
      for (let i = 0; i < interleaved.length; i++, offset += 2) {
        const sample = Math.max(-1, Math.min(1, interleaved[i]));
        view.setInt16(offset, sample < 0 ? sample * 0x8000 : sample * 0x7fff, true);
      }
    } else if (bitDepth === 24) {
      for (let i = 0; i < interleaved.length; i++, offset += 3) {
        const sample = Math.max(-1, Math.min(1, interleaved[i]));
        const val = sample < 0 ? sample * 0x800000 : sample * 0x7fffff;
        const intVal = Math.floor(val);
        view.setUint8(offset, intVal & 0xff);
        view.setUint8(offset + 1, (intVal >> 8) & 0xff);
        view.setUint8(offset + 2, (intVal >> 16) & 0xff);
      }
    }

    return buffer;
  }

  /**
   * Write string to DataView
   */
  private static writeString(view: DataView, offset: number, str: string): void {
    for (let i = 0; i < str.length; i++) {
      view.setUint8(offset + i, str.charCodeAt(i));
    }
  }

  /**
   * Format file size for display
   */
  static formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
}

/**
 * Create a dummy audio buffer for testing export
 * Generates a simple sine wave
 */
export async function createTestAudioBuffer(
  duration: number = 5,
  sampleRate: number = 44100
): Promise<AudioBuffer> {
  const ctx = Tone.getContext().rawContext;
  const buffer = ctx.createBuffer(2, sampleRate * duration, sampleRate);

  // Generate a simple sine wave for testing
  const frequency = 440; // A4
  for (let channel = 0; channel < 2; channel++) {
    const data = buffer.getChannelData(channel);
    for (let i = 0; i < data.length; i++) {
      const t = i / sampleRate;
      data[i] = Math.sin(2 * Math.PI * frequency * t) * 0.3;
      // Add some envelope
      const env = Math.min(1, Math.min(t * 10, (duration - t) * 10));
      data[i] *= env;
    }
  }

  return buffer;
}

export default AudioExporter;
