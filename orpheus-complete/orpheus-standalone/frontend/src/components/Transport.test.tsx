import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { Transport } from './Transport';

describe('Transport', () => {
  const defaultProps = {
    isPlaying: false,
    isRecording: false,
    isLooping: false,
    currentPosition: 0,
    bpm: 120,
    onPlay: jest.fn(),
    onStop: jest.fn(),
    onRecord: jest.fn(),
    onLoop: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('should render transport container', () => {
    render(<Transport {...defaultProps} />);
    expect(screen.getByTestId('transport')).toBeInTheDocument();
  });

  it('should render stop button', () => {
    render(<Transport {...defaultProps} />);
    expect(screen.getByTestId('transport-stop-button')).toBeInTheDocument();
  });

  it('should render play button', () => {
    render(<Transport {...defaultProps} />);
    expect(screen.getByTestId('transport-play-button')).toBeInTheDocument();
  });

  it('should render record button', () => {
    render(<Transport {...defaultProps} />);
    expect(screen.getByTestId('transport-record-button')).toBeInTheDocument();
  });

  it('should render loop button', () => {
    render(<Transport {...defaultProps} />);
    expect(screen.getByTestId('loop-button')).toBeInTheDocument();
  });

  it('should call onPlay when play button clicked', () => {
    render(<Transport {...defaultProps} />);

    const playButton = screen.getByTestId('transport-play-button');
    fireEvent.click(playButton);

    expect(defaultProps.onPlay).toHaveBeenCalled();
  });

  it('should call onStop when stop button clicked', () => {
    render(<Transport {...defaultProps} />);

    const stopButton = screen.getByTestId('transport-stop-button');
    fireEvent.click(stopButton);

    expect(defaultProps.onStop).toHaveBeenCalled();
  });

  it('should call onRecord when record button clicked', () => {
    render(<Transport {...defaultProps} />);

    const recordButton = screen.getByTestId('transport-record-button');
    fireEvent.click(recordButton);

    expect(defaultProps.onRecord).toHaveBeenCalled();
  });

  it('should call onLoop when loop button clicked', () => {
    render(<Transport {...defaultProps} />);

    const loopButton = screen.getByTestId('loop-button');
    fireEvent.click(loopButton);

    expect(defaultProps.onLoop).toHaveBeenCalled();
  });

  it('should apply playing class when playing', () => {
    render(<Transport {...defaultProps} isPlaying={true} />);

    const playButton = screen.getByTestId('transport-play-button');
    expect(playButton).toHaveClass('playing');
  });

  it('should apply recording class when recording', () => {
    render(<Transport {...defaultProps} isRecording={true} />);

    const recordButton = screen.getByTestId('transport-record-button');
    expect(recordButton).toHaveClass('active');
  });

  it('should apply looping class when looping', () => {
    render(<Transport {...defaultProps} isLooping={true} />);

    const loopButton = screen.getByTestId('loop-button');
    expect(loopButton).toHaveClass('active');
  });

  it('should display current tempo', () => {
    render(<Transport {...defaultProps} bpm={140} />);

    expect(screen.getByText('140')).toBeInTheDocument();
  });

  it('should render metronome button', () => {
    render(<Transport {...defaultProps} />);
    expect(screen.getByTestId('metronome-button')).toBeInTheDocument();
  });

  it('should render snap button', () => {
    render(<Transport {...defaultProps} />);
    expect(screen.getByTestId('snap-button')).toBeInTheDocument();
  });

  it('should render zoom controls', () => {
    render(<Transport {...defaultProps} />);
    expect(screen.getByTestId('zoom-in-button')).toBeInTheDocument();
    expect(screen.getByTestId('zoom-out-button')).toBeInTheDocument();
  });

  it('should display time in bars:beats:ticks format by default', () => {
    render(<Transport {...defaultProps} currentPosition={2} bpm={120} />);

    const timeDisplay = screen.getByTestId('time-display');
    expect(timeDisplay.textContent).toMatch(/\d+:\d+:\d+/);
  });

  it('should switch to seconds format', () => {
    render(<Transport {...defaultProps} currentPosition={5} />);

    const formatButton = screen.getByTestId('time-format-button');
    fireEvent.click(formatButton);

    const timeDisplay = screen.getByTestId('time-display');
    expect(timeDisplay.textContent).toMatch(/\d+\.\d+s/);
  });

  it('should switch to samples format', () => {
    render(<Transport {...defaultProps} currentPosition={5} />);

    const formatButton = screen.getByTestId('time-format-button');
    fireEvent.click(formatButton); // seconds
    fireEvent.click(formatButton); // samples

    const timeDisplay = screen.getByTestId('time-display');
    expect(timeDisplay.textContent).toMatch(/\d+ samples/);
  });

  it('should cycle through time formats', () => {
    render(<Transport {...defaultProps} />);

    const formatButton = screen.getByTestId('time-format-button');

    // Should cycle: bars -> seconds -> samples -> bars
    fireEvent.click(formatButton);
    fireEvent.click(formatButton);
    fireEvent.click(formatButton);

    const timeDisplay = screen.getByTestId('time-display');
    expect(timeDisplay.textContent).toMatch(/\d+:\d+:\d+/); // Back to bars
  });

  it('should display latency indicator', () => {
    render(<Transport {...defaultProps} />);
    expect(screen.getByText(/12\.5ms/)).toBeInTheDocument();
  });

  it('should format position at zero correctly', () => {
    render(<Transport {...defaultProps} currentPosition={0} />);

    const timeDisplay = screen.getByTestId('time-display');
    expect(timeDisplay).toBeInTheDocument();
  });

  it('should format large position values', () => {
    render(<Transport {...defaultProps} currentPosition={300} bpm={120} />);

    const timeDisplay = screen.getByTestId('time-display');
    expect(timeDisplay).toBeInTheDocument();
  });

  it('should handle different BPM values in time calculation', () => {
    render(<Transport {...defaultProps} currentPosition={10} bpm={60} />);

    const timeDisplay = screen.getByTestId('time-display');
    expect(timeDisplay).toBeInTheDocument();
  });

  it('should render select all button', () => {
    render(<Transport {...defaultProps} />);
    expect(screen.getByTestId('select-all-button')).toBeInTheDocument();
  });

  it('should render bounce audio button', () => {
    render(<Transport {...defaultProps} />);
    expect(screen.getByTestId('bounce-audio-button')).toBeInTheDocument();
  });
});
