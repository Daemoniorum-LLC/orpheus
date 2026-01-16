import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { Track } from './Track';

describe('Track', () => {
  const mockTrack = {
    id: 'track-1',
    name: 'Test Track',
    volume: 0.8,
    pan: 0,
    muted: false,
    soloed: false,
    recordArmed: false,
    peakLevel: 0.6,
    rmsLevel: 0.4,
  };

  const mockOnUpdate = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('should render track container', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);
    expect(screen.getByTestId('track-item')).toBeInTheDocument();
  });

  it('should display track name', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);
    expect(screen.getByText('Test Track')).toBeInTheDocument();
  });

  it('should render mute button', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);
    expect(screen.getByTestId('track-mute-button')).toBeInTheDocument();
  });

  it('should render solo button', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);
    expect(screen.getByTestId('track-solo-button')).toBeInTheDocument();
  });

  it('should render record arm button', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);
    expect(screen.getByTestId('track-record-arm')).toBeInTheDocument();
  });

  it('should render volume slider', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);
    expect(screen.getByTestId('track-volume-slider')).toBeInTheDocument();
  });

  it('should render pan slider', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);
    expect(screen.getByTestId('track-pan-slider')).toBeInTheDocument();
  });

  it('should render peak meter', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);
    expect(screen.getByTestId('peak-meter')).toBeInTheDocument();
  });

  it('should render RMS meter', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);
    expect(screen.getByTestId('rms-meter')).toBeInTheDocument();
  });

  it('should toggle mute on button click', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);

    const muteButton = screen.getByTestId('track-mute-button');
    fireEvent.click(muteButton);

    expect(mockOnUpdate).toHaveBeenCalledWith({ muted: true });
  });

  it('should toggle solo on button click', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);

    const soloButton = screen.getByTestId('track-solo-button');
    fireEvent.click(soloButton);

    expect(mockOnUpdate).toHaveBeenCalledWith({ soloed: true });
  });

  it('should toggle record arm on button click', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);

    const recordArmButton = screen.getByTestId('track-record-arm');
    fireEvent.click(recordArmButton);

    expect(mockOnUpdate).toHaveBeenCalledWith({ recordArmed: true });
  });

  it('should update volume on slider change', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);

    const volumeSlider = screen.getByTestId('track-volume-slider');
    fireEvent.change(volumeSlider, { target: { value: '0.5' } });

    expect(mockOnUpdate).toHaveBeenCalledWith({ volume: 0.5 });
  });

  it('should update pan on slider change', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);

    const panSlider = screen.getByTestId('track-pan-slider');
    fireEvent.change(panSlider, { target: { value: '0.3' } });

    expect(mockOnUpdate).toHaveBeenCalledWith({ pan: 0.3 });
  });

  it('should display volume value correctly', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);

    const volumeSlider = screen.getByTestId('track-volume-slider') as HTMLInputElement;
    expect(volumeSlider.value).toBe('0.8');
  });

  it('should display pan value correctly', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);

    const panSlider = screen.getByTestId('track-pan-slider') as HTMLInputElement;
    expect(panSlider.value).toBe('0');
  });

  it('should apply muted class when muted', () => {
    const mutedTrack = { ...mockTrack, muted: true };
    render(<Track track={mutedTrack} onUpdate={mockOnUpdate} />);

    const muteButton = screen.getByTestId('track-mute-button');
    expect(muteButton).toHaveClass('active');
  });

  it('should apply soloed class when soloed', () => {
    const soloedTrack = { ...mockTrack, soloed: true };
    render(<Track track={soloedTrack} onUpdate={mockOnUpdate} />);

    const soloButton = screen.getByTestId('track-solo-button');
    expect(soloButton).toHaveClass('active');
  });

  it('should apply record armed class when record armed', () => {
    const armedTrack = { ...mockTrack, recordArmed: true };
    render(<Track track={armedTrack} onUpdate={mockOnUpdate} />);

    const recordArmButton = screen.getByTestId('track-record-arm');
    expect(recordArmButton).toHaveClass('active');
  });

  it('should display peak level meter correctly', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);

    const peakMeter = screen.getByTestId('peak-meter');
    const style = window.getComputedStyle(peakMeter);

    // Peak level of 0.6 should result in 60% height
    expect(peakMeter.style.height).toBe('60%');
  });

  it('should display RMS level meter correctly', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);

    const rmsMeter = screen.getByTestId('rms-meter');

    // RMS level of 0.4 should result in 40% height
    expect(rmsMeter.style.height).toBe('40%');
  });

  it('should handle zero peak level', () => {
    const zeroLevelTrack = { ...mockTrack, peakLevel: 0 };
    render(<Track track={zeroLevelTrack} onUpdate={mockOnUpdate} />);

    const peakMeter = screen.getByTestId('peak-meter');
    expect(peakMeter.style.height).toBe('0%');
  });

  it('should handle max peak level', () => {
    const maxLevelTrack = { ...mockTrack, peakLevel: 1 };
    render(<Track track={maxLevelTrack} onUpdate={mockOnUpdate} />);

    const peakMeter = screen.getByTestId('peak-meter');
    expect(peakMeter.style.height).toBe('100%');
  });

  it('should handle negative pan values', () => {
    const leftPannedTrack = { ...mockTrack, pan: -0.5 };
    render(<Track track={leftPannedTrack} onUpdate={mockOnUpdate} />);

    const panSlider = screen.getByTestId('track-pan-slider') as HTMLInputElement;
    expect(panSlider.value).toBe('-0.5');
  });

  it('should handle positive pan values', () => {
    const rightPannedTrack = { ...mockTrack, pan: 0.7 };
    render(<Track track={rightPannedTrack} onUpdate={mockOnUpdate} />);

    const panSlider = screen.getByTestId('track-pan-slider') as HTMLInputElement;
    expect(panSlider.value).toBe('0.7');
  });

  it('should handle min volume', () => {
    const minVolumeTrack = { ...mockTrack, volume: 0 };
    render(<Track track={minVolumeTrack} onUpdate={mockOnUpdate} />);

    const volumeSlider = screen.getByTestId('track-volume-slider') as HTMLInputElement;
    expect(volumeSlider.value).toBe('0');
  });

  it('should handle max volume', () => {
    const maxVolumeTrack = { ...mockTrack, volume: 1 };
    render(<Track track={maxVolumeTrack} onUpdate={mockOnUpdate} />);

    const volumeSlider = screen.getByTestId('track-volume-slider') as HTMLInputElement;
    expect(volumeSlider.value).toBe('1');
  });

  it('should call onUpdate only once per interaction', () => {
    render(<Track track={mockTrack} onUpdate={mockOnUpdate} />);

    const muteButton = screen.getByTestId('track-mute-button');
    fireEvent.click(muteButton);

    expect(mockOnUpdate).toHaveBeenCalledTimes(1);
  });

  it('should unmute when clicking mute button on muted track', () => {
    const mutedTrack = { ...mockTrack, muted: true };
    render(<Track track={mutedTrack} onUpdate={mockOnUpdate} />);

    const muteButton = screen.getByTestId('track-mute-button');
    fireEvent.click(muteButton);

    expect(mockOnUpdate).toHaveBeenCalledWith({ muted: false });
  });

  it('should unsolo when clicking solo button on soloed track', () => {
    const soloedTrack = { ...mockTrack, soloed: true };
    render(<Track track={soloedTrack} onUpdate={mockOnUpdate} />);

    const soloButton = screen.getByTestId('track-solo-button');
    fireEvent.click(soloButton);

    expect(mockOnUpdate).toHaveBeenCalledWith({ soloed: false });
  });

  it('should disarm when clicking record arm on armed track', () => {
    const armedTrack = { ...mockTrack, recordArmed: true };
    render(<Track track={armedTrack} onUpdate={mockOnUpdate} />);

    const recordArmButton = screen.getByTestId('track-record-arm');
    fireEvent.click(recordArmButton);

    expect(mockOnUpdate).toHaveBeenCalledWith({ recordArmed: false });
  });
});
