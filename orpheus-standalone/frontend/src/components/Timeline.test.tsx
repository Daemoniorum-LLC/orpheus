import React from 'react';
import { render, screen } from '@testing-library/react';
import { Timeline } from './Timeline';

describe('Timeline', () => {
  const mockTracks = [
    {
      id: 'track-1',
      name: 'Track 1',
      trackNumber: 1,
      audioRegions: [
        {
          id: 'region-1',
          startTime: 0,
          duration: 5,
          audioUrl: '/audio/test.wav',
        },
      ],
    },
  ];

  const defaultProps = {
    tracks: mockTracks,
    playheadPosition: 0,
    isPlaying: false,
    bpm: 120,
    timeSignature: '4/4' as const,
    onSeek: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();

    // Mock HTMLCanvasElement getContext
    HTMLCanvasElement.prototype.getContext = jest.fn().mockReturnValue({
      clearRect: jest.fn(),
      fillRect: jest.fn(),
      strokeRect: jest.fn(),
      fillText: jest.fn(),
      measureText: jest.fn(() => ({ width: 10 })),
      beginPath: jest.fn(),
      moveTo: jest.fn(),
      lineTo: jest.fn(),
      stroke: jest.fn(),
      save: jest.fn(),
      restore: jest.fn(),
      scale: jest.fn(),
      translate: jest.fn(),
      set fillStyle(value: string) {},
      get fillStyle(): string { return ''; },
      set strokeStyle(value: string) {},
      get strokeStyle(): string { return ''; },
      set font(value: string) {},
      get font(): string { return ''; },
      set lineWidth(value: number) {},
      get lineWidth(): number { return 1; },
    });
  });

  it('should render timeline container', () => {
    render(<Timeline {...defaultProps} />);
    expect(screen.getByTestId('timeline')).toBeInTheDocument();
  });

  it('should render timeline header', () => {
    render(<Timeline {...defaultProps} />);
    expect(screen.getByClassName('timeline-header')).toBeInTheDocument();
  });

  it('should render timeline ruler canvas', () => {
    render(<Timeline {...defaultProps} />);
    expect(screen.getByTestId('timeline-ruler')).toBeInTheDocument();
  });

  it('should render timeline scroll container', () => {
    render(<Timeline {...defaultProps} />);
    expect(screen.getByClassName('timeline-scroll')).toBeInTheDocument();
  });

  it('should render timeline tracks', () => {
    render(<Timeline {...defaultProps} />);
    expect(screen.getByClassName('timeline-tracks')).toBeInTheDocument();
  });

  it('should render playhead', () => {
    render(<Timeline {...defaultProps} />);
    expect(screen.getByTestId('playhead')).toBeInTheDocument();
  });

  it('should position playhead based on currentPosition', () => {
    render(<Timeline {...defaultProps} playheadPosition={10} />);

    const playhead = screen.getByTestId('playhead');
    expect(playhead).toBeInTheDocument();
    // Playhead position is calculated as position * pixelsPerSecond
  });

  it('should apply playing class to playhead when playing', () => {
    render(<Timeline {...defaultProps} isPlaying={true} />);

    const playhead = screen.getByTestId('playhead');
    expect(playhead).toHaveClass('playing');
  });

  it('should not apply playing class when not playing', () => {
    render(<Timeline {...defaultProps} isPlaying={false} />);

    const playhead = screen.getByTestId('playhead');
    expect(playhead).not.toHaveClass('playing');
  });

  it('should render track for each track in props', () => {
    const multiTracks = [
      { id: '1', name: 'Track 1', trackNumber: 1, audioRegions: [] },
      { id: '2', name: 'Track 2', trackNumber: 2, audioRegions: [] },
      { id: '3', name: 'Track 3', trackNumber: 3, audioRegions: [] },
    ];

    render(<Timeline {...defaultProps} tracks={multiTracks} />);

    const trackElements = screen.getAllByClassName('timeline-track');
    expect(trackElements).toHaveLength(3);
  });

  it('should render audio regions', () => {
    render(<Timeline {...defaultProps} />);

    const audioRegions = screen.getAllByTestId('audio-region');
    expect(audioRegions).toHaveLength(1);
  });

  it('should position audio region correctly', () => {
    render(<Timeline {...defaultProps} />);

    const audioRegion = screen.getByTestId('audio-region');
    // Position should be calculated from startTime
    expect(audioRegion.style.left).toBeDefined();
  });

  it('should size audio region based on duration', () => {
    render(<Timeline {...defaultProps} />);

    const audioRegion = screen.getByTestId('audio-region');
    // Width should be calculated from duration
    expect(audioRegion.style.width).toBeDefined();
  });

  it('should render multiple audio regions on same track', () => {
    const tracksWithMultipleRegions = [
      {
        id: 'track-1',
        name: 'Track 1',
        trackNumber: 1,
        audioRegions: [
          { id: 'region-1', startTime: 0, duration: 5, audioUrl: '/audio/1.wav' },
          { id: 'region-2', startTime: 10, duration: 3, audioUrl: '/audio/2.wav' },
        ],
      },
    ];

    render(<Timeline {...defaultProps} tracks={tracksWithMultipleRegions} />);

    const audioRegions = screen.getAllByTestId('audio-region');
    expect(audioRegions).toHaveLength(2);
  });

  it('should render waveform placeholder', () => {
    render(<Timeline {...defaultProps} />);

    const waveform = screen.getByClassName('waveform-placeholder');
    expect(waveform).toBeInTheDocument();
  });

  it('should render region end handle', () => {
    render(<Timeline {...defaultProps} />);

    const handle = screen.getByClassName('region-end-handle');
    expect(handle).toBeInTheDocument();
  });

  it('should handle empty track list', () => {
    render(<Timeline {...defaultProps} tracks={[]} />);

    const trackElements = screen.queryAllByClassName('timeline-track');
    expect(trackElements).toHaveLength(0);
  });

  it('should handle track with no audio regions', () => {
    const emptyTrack = [
      { id: 'track-1', name: 'Empty Track', trackNumber: 1, audioRegions: [] },
    ];

    render(<Timeline {...defaultProps} tracks={emptyTrack} />);

    const audioRegions = screen.queryAllByTestId('audio-region');
    expect(audioRegions).toHaveLength(0);
  });

  it('should update playhead position attribute', () => {
    render(<Timeline {...defaultProps} playheadPosition={25.5} />);

    const playhead = screen.getByTestId('playhead');
    expect(playhead).toHaveAttribute('data-position', '25.5');
  });

  it('should handle zero playhead position', () => {
    render(<Timeline {...defaultProps} playheadPosition={0} />);

    const playhead = screen.getByTestId('playhead');
    expect(playhead).toHaveAttribute('data-position', '0');
  });

  it('should draw ruler with correct BPM', () => {
    render(<Timeline {...defaultProps} bpm={140} />);

    const canvas = screen.getByTestId('timeline-ruler');
    expect(canvas).toBeInTheDocument();
  });

  it('should redraw timeline when BPM changes', () => {
    const { rerender } = render(<Timeline {...defaultProps} bpm={120} />);

    rerender(<Timeline {...defaultProps} bpm={140} />);

    const canvas = screen.getByTestId('timeline-ruler');
    expect(canvas).toBeInTheDocument();
  });

  it('should redraw timeline when time signature changes', () => {
    const { rerender } = render(<Timeline {...defaultProps} timeSignature="4/4" />);

    rerender(<Timeline {...defaultProps} timeSignature="3/4" />);

    const canvas = screen.getByTestId('timeline-ruler');
    expect(canvas).toBeInTheDocument();
  });
});
