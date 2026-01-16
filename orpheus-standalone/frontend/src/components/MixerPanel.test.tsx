import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { MixerPanel } from './MixerPanel';

describe('MixerPanel', () => {
  const mockTracks = [
    {
      id: 'track-1',
      name: 'Drums',
      volume: 0.8,
      pan: 0,
      muted: false,
      soloed: false,
      peakLevel: 0.6,
      rmsLevel: 0.4,
      effects: [],
    },
    {
      id: 'track-2',
      name: 'Bass',
      volume: 0.7,
      pan: 0.5,
      muted: false,
      soloed: false,
      peakLevel: 0.5,
      rmsLevel: 0.3,
      effects: [
        {
          id: 'effect-1',
          type: 'compressor',
          name: 'Compressor',
          enabled: true,
          parameters: {},
        },
      ],
    },
  ];

  const mockOnUpdateTrack = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('should render mixer panel', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);
    expect(screen.getByTestId('mixer-panel')).toBeInTheDocument();
  });

  it('should render mixer header', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);
    expect(screen.getByText('Mixer')).toBeInTheDocument();
  });

  it('should render master bus button', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);
    expect(screen.getByTestId('master-bus')).toBeInTheDocument();
  });

  it('should render mixer channels for all tracks', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const channels = screen.getAllByTestId('mixer-channel');
    expect(channels).toHaveLength(2);
  });

  it('should display track names in channels', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    expect(screen.getByText('Drums')).toBeInTheDocument();
    expect(screen.getByText('Bass')).toBeInTheDocument();
  });

  it('should render effects button for each track', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const effectsButtons = screen.getAllByTestId('track-effects-button');
    expect(effectsButtons).toHaveLength(2);
  });

  it('should display effect count on effects button', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const effectsButtons = screen.getAllByTestId('track-effects-button');
    expect(effectsButtons[0]).toHaveTextContent('FX (0)');
    expect(effectsButtons[1]).toHaveTextContent('FX (1)');
  });

  it('should render peak meters', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const peakMeters = screen.getAllByTestId('peak-meter');
    expect(peakMeters).toHaveLength(2);
  });

  it('should render RMS meters', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const rmsMeters = screen.getAllByTestId('rms-meter');
    expect(rmsMeters).toHaveLength(2);
  });

  it('should set meter heights based on levels', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const peakMeters = screen.getAllByTestId('peak-meter');
    expect(peakMeters[0].style.height).toBe('60%'); // 0.6 * 100%
  });

  it('should render pan sliders', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const panSliders = screen.getAllByTestId('track-pan-slider');
    expect(panSliders).toHaveLength(2);
  });

  it('should render volume sliders', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const volumeSliders = screen.getAllByTestId('track-volume-slider');
    expect(volumeSliders).toHaveLength(2);
  });

  it('should update volume when slider changed', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const volumeSlider = screen.getAllByTestId('track-volume-slider')[0];
    fireEvent.change(volumeSlider, { target: { value: '0.5' } });

    expect(mockOnUpdateTrack).toHaveBeenCalledWith('track-1', { volume: 0.5 });
  });

  it('should update pan when slider changed', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const panSlider = screen.getAllByTestId('track-pan-slider')[0];
    fireEvent.change(panSlider, { target: { value: '0.3' } });

    expect(mockOnUpdateTrack).toHaveBeenCalledWith('track-1', { pan: 0.3 });
  });

  it('should show pan label for center', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    expect(screen.getByText('C')).toBeInTheDocument();
  });

  it('should show pan label for left', () => {
    const leftPannedTrack = [{ ...mockTracks[0], pan: -0.5 }];
    render(<MixerPanel tracks={leftPannedTrack} onUpdateTrack={mockOnUpdateTrack} />);

    expect(screen.getByText('L50')).toBeInTheDocument();
  });

  it('should show pan label for right', () => {
    const rightPannedTrack = [{ ...mockTracks[0], pan: 0.7 }];
    render(<MixerPanel tracks={rightPannedTrack} onUpdateTrack={mockOnUpdateTrack} />);

    expect(screen.getByText('R70')).toBeInTheDocument();
  });

  it('should display volume percentage', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    expect(screen.getByText('80%')).toBeInTheDocument(); // track-1 volume
    expect(screen.getByText('70%')).toBeInTheDocument(); // track-2 volume
  });

  it('should open effects panel when effects button clicked', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const effectsButton = screen.getAllByTestId('track-effects-button')[0];
    fireEvent.click(effectsButton);

    expect(screen.getByTestId('effects-panel')).toBeInTheDocument();
  });

  it('should close effects panel when close button clicked', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const effectsButton = screen.getAllByTestId('track-effects-button')[0];
    fireEvent.click(effectsButton);

    const closeButton = screen.getByText('×');
    fireEvent.click(closeButton);

    expect(screen.queryByTestId('effects-panel')).not.toBeInTheDocument();
  });

  it('should display track name in effects panel header', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const effectsButton = screen.getAllByTestId('track-effects-button')[0];
    fireEvent.click(effectsButton);

    expect(screen.getByText(/Effects - Drums/)).toBeInTheDocument();
  });

  it('should show add effect button in effects panel', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const effectsButton = screen.getAllByTestId('track-effects-button')[0];
    fireEvent.click(effectsButton);

    expect(screen.getByTestId('add-effect-button')).toBeInTheDocument();
  });

  it('should show effect menu options', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const effectsButton = screen.getAllByTestId('track-effects-button')[0];
    fireEvent.click(effectsButton);

    expect(screen.getByTestId('effect-eq')).toBeInTheDocument();
    expect(screen.getByTestId('effect-compressor')).toBeInTheDocument();
    expect(screen.getByTestId('effect-reverb')).toBeInTheDocument();
    expect(screen.getByTestId('effect-delay')).toBeInTheDocument();
    expect(screen.getByTestId('effect-distortion')).toBeInTheDocument();
    expect(screen.getByTestId('effect-limiter')).toBeInTheDocument();
  });

  it('should add effect when effect button clicked', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const effectsButton = screen.getAllByTestId('track-effects-button')[0];
    fireEvent.click(effectsButton);

    const eqButton = screen.getByTestId('effect-eq');
    fireEvent.click(eqButton);

    expect(mockOnUpdateTrack).toHaveBeenCalled();
  });

  it('should display existing effects', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const effectChips = screen.getAllByTestId('effect-chip');
    expect(effectChips).toHaveLength(1);
  });

  it('should toggle effect bypass', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const effectBypass = screen.getAllByTestId('effect-bypass')[0];
    fireEvent.click(effectBypass);

    expect(mockOnUpdateTrack).toHaveBeenCalled();
  });

  it('should remove effect', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const removeButton = screen.getByTestId('remove-effect');
    fireEvent.click(removeButton);

    expect(mockOnUpdateTrack).toHaveBeenCalled();
  });

  it('should show master bus panel when master button clicked', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const masterButton = screen.getByTestId('master-bus');
    fireEvent.click(masterButton);

    expect(screen.getByTestId('master-bus-panel')).toBeInTheDocument();
  });

  it('should close master bus panel', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const masterButton = screen.getByTestId('master-bus');
    fireEvent.click(masterButton);

    const closeButton = screen.getAllByText('×')[0];
    fireEvent.click(closeButton);

    expect(screen.queryByTestId('master-bus-panel')).not.toBeInTheDocument();
  });

  it('should highlight selected channel', () => {
    render(<MixerPanel tracks={mockTracks} onUpdateTrack={mockOnUpdateTrack} />);

    const effectsButton = screen.getAllByTestId('track-effects-button')[0];
    fireEvent.click(effectsButton);

    const channels = screen.getAllByTestId('mixer-channel');
    expect(channels[0]).toHaveClass('selected');
  });

  it('should handle disabled effects', () => {
    const tracksWithDisabledEffect = [
      {
        ...mockTracks[0],
        effects: [
          { id: 'e1', type: 'eq', name: 'EQ', enabled: false, parameters: {} },
        ],
      },
    ];

    render(<MixerPanel tracks={tracksWithDisabledEffect} onUpdateTrack={mockOnUpdateTrack} />);

    const effectChip = screen.getByTestId('effect-chip');
    expect(effectChip).toHaveClass('disabled');
  });
});
