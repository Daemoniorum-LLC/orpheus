import { expect, afterEach, vi } from 'vitest';
import { cleanup } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';

// Cleanup after each test
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

// Mock Web Audio API
global.AudioContext = vi.fn().mockImplementation(() => ({
  createOscillator: vi.fn(),
  createGain: vi.fn(),
  createAnalyser: vi.fn(),
  createBiquadFilter: vi.fn(),
  createDynamicsCompressor: vi.fn(),
  createConvolver: vi.fn(),
  createDelay: vi.fn(),
  destination: {},
  currentTime: 0,
  sampleRate: 44100,
  state: 'running',
  resume: vi.fn(),
  suspend: vi.fn(),
  close: vi.fn(),
})) as any;

// Mock OfflineAudioContext
global.OfflineAudioContext = vi.fn().mockImplementation(() => ({
  ...global.AudioContext(),
  startRendering: vi.fn().mockResolvedValue({
    getChannelData: vi.fn().mockReturnValue(new Float32Array(1024)),
    length: 1024,
    numberOfChannels: 2,
    sampleRate: 44100,
  }),
})) as any;

// Mock MediaStream API
global.navigator.mediaDevices = {
  getUserMedia: vi.fn().mockResolvedValue({
    getTracks: vi.fn().mockReturnValue([]),
    getAudioTracks: vi.fn().mockReturnValue([]),
    getVideoTracks: vi.fn().mockReturnValue([]),
  }),
  enumerateDevices: vi.fn().mockResolvedValue([]),
} as any;

// Mock File API
global.FileReader = class FileReader {
  readAsArrayBuffer = vi.fn();
  readAsText = vi.fn();
  readAsDataURL = vi.fn();
  addEventListener = vi.fn();
  removeEventListener = vi.fn();
  result: any = null;
} as any;

// Mock Blob
global.Blob = class Blob {
  constructor(public parts: any[], public options?: any) {}
  size = 0;
  type = '';
  arrayBuffer = vi.fn().mockResolvedValue(new ArrayBuffer(0));
  slice = vi.fn().mockReturnThis();
  stream = vi.fn();
  text = vi.fn().mockResolvedValue('');
} as any;

// Mock URL.createObjectURL
global.URL.createObjectURL = vi.fn(() => 'blob:mock-url');
global.URL.revokeObjectURL = vi.fn();
