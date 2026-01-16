import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { EffectsProcessor } from './EffectsProcessor';

describe('EffectsProcessor', () => {
  const mockOnUpdateParameter = jest.fn();
  const mockOnToggleBypass = jest.fn();
  const mockOnRemove = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('EQ Effect', () => {
    const eqEffect = {
      id: 'eq-1',
      type: 'eq',
      name: 'EQ',
      enabled: true,
      parameters: {
        band0Freq: 100,
        band0Gain: 0,
        band0Q: 1,
        band1Freq: 500,
        band1Gain: 3,
        band1Q: 1.5,
      },
    };

    it('should render EQ effect', () => {
      render(
        <EffectsProcessor
          effect={eqEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByText('EQ')).toBeInTheDocument();
    });

    it('should render all 4 band controls', () => {
      render(
        <EffectsProcessor
          effect={eqEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByTestId('eq-band-0-freq')).toBeInTheDocument();
      expect(screen.getByTestId('eq-band-1-freq')).toBeInTheDocument();
      expect(screen.getByTestId('eq-band-2-freq')).toBeInTheDocument();
      expect(screen.getByTestId('eq-band-3-freq')).toBeInTheDocument();
    });

    it('should display EQ visualizer', () => {
      render(
        <EffectsProcessor
          effect={eqEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByTestId('eq-visualizer')).toBeInTheDocument();
    });

    it('should update EQ parameter', () => {
      render(
        <EffectsProcessor
          effect={eqEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      const freqSlider = screen.getByTestId('eq-band-0-freq');
      fireEvent.change(freqSlider, { target: { value: '2000' } });

      expect(mockOnUpdateParameter).toHaveBeenCalledWith('eq-band-0-freq', 2000);
    });
  });

  describe('Compressor Effect', () => {
    const compressorEffect = {
      id: 'comp-1',
      type: 'compressor',
      name: 'Compressor',
      enabled: true,
      parameters: {
        threshold: -20,
        ratio: 4,
        attack: 10,
        release: 100,
        makeupGain: 0,
      },
    };

    it('should render compressor effect', () => {
      render(
        <EffectsProcessor
          effect={compressorEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByText('Compressor')).toBeInTheDocument();
    });

    it('should render compressor parameters', () => {
      render(
        <EffectsProcessor
          effect={compressorEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByTestId('compressor-threshold')).toBeInTheDocument();
      expect(screen.getByTestId('compressor-ratio')).toBeInTheDocument();
      expect(screen.getByTestId('compressor-attack')).toBeInTheDocument();
      expect(screen.getByTestId('compressor-release')).toBeInTheDocument();
      expect(screen.getByTestId('compressor-makeup')).toBeInTheDocument();
    });

    it('should display gain reduction meter', () => {
      render(
        <EffectsProcessor
          effect={compressorEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByTestId('gain-reduction-meter')).toBeInTheDocument();
    });

    it('should update compressor parameter', () => {
      render(
        <EffectsProcessor
          effect={compressorEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      const thresholdSlider = screen.getByTestId('compressor-threshold');
      fireEvent.change(thresholdSlider, { target: { value: '-12' } });

      expect(mockOnUpdateParameter).toHaveBeenCalledWith('compressor-threshold', -12);
    });
  });

  describe('Reverb Effect', () => {
    const reverbEffect = {
      id: 'reverb-1',
      type: 'reverb',
      name: 'Reverb',
      enabled: true,
      parameters: {
        roomSize: 0.5,
        damping: 0.5,
        wet: 0.3,
        dry: 0.7,
        predelay: 0,
      },
    };

    it('should render reverb effect', () => {
      render(
        <EffectsProcessor
          effect={reverbEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByText('Reverb')).toBeInTheDocument();
    });

    it('should render reverb parameters', () => {
      render(
        <EffectsProcessor
          effect={reverbEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByTestId('reverb-room-size')).toBeInTheDocument();
      expect(screen.getByTestId('reverb-damping')).toBeInTheDocument();
      expect(screen.getByTestId('reverb-wet')).toBeInTheDocument();
      expect(screen.getByTestId('reverb-dry')).toBeInTheDocument();
      expect(screen.getByTestId('reverb-predelay')).toBeInTheDocument();
    });
  });

  describe('Delay Effect', () => {
    const delayEffect = {
      id: 'delay-1',
      type: 'delay',
      name: 'Delay',
      enabled: true,
      parameters: {
        delayTime: 500,
        feedback: 0.3,
        wet: 0.3,
        dry: 0.7,
      },
    };

    it('should render delay effect', () => {
      render(
        <EffectsProcessor
          effect={delayEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByText('Delay')).toBeInTheDocument();
    });

    it('should render delay parameters', () => {
      render(
        <EffectsProcessor
          effect={delayEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByTestId('delay-time')).toBeInTheDocument();
      expect(screen.getByTestId('delay-feedback')).toBeInTheDocument();
      expect(screen.getByTestId('delay-wet')).toBeInTheDocument();
      expect(screen.getByTestId('delay-dry')).toBeInTheDocument();
    });
  });

  describe('Distortion Effect', () => {
    const distortionEffect = {
      id: 'dist-1',
      type: 'distortion',
      name: 'Distortion',
      enabled: true,
      parameters: {
        drive: 5,
        tone: 0.5,
        output: 0,
      },
    };

    it('should render distortion effect', () => {
      render(
        <EffectsProcessor
          effect={distortionEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByText('Distortion')).toBeInTheDocument();
    });

    it('should render distortion parameters', () => {
      render(
        <EffectsProcessor
          effect={distortionEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByTestId('distortion-drive')).toBeInTheDocument();
      expect(screen.getByTestId('distortion-tone')).toBeInTheDocument();
      expect(screen.getByTestId('distortion-output')).toBeInTheDocument();
    });
  });

  describe('Limiter Effect', () => {
    const limiterEffect = {
      id: 'lim-1',
      type: 'limiter',
      name: 'Limiter',
      enabled: true,
      parameters: {
        threshold: -1,
        release: 50,
      },
    };

    it('should render limiter effect', () => {
      render(
        <EffectsProcessor
          effect={limiterEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByText('Limiter')).toBeInTheDocument();
    });

    it('should render limiter parameters', () => {
      render(
        <EffectsProcessor
          effect={limiterEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByTestId('limiter-threshold')).toBeInTheDocument();
      expect(screen.getByTestId('limiter-release')).toBeInTheDocument();
    });
  });

  describe('Common functionality', () => {
    const testEffect = {
      id: 'test-1',
      type: 'compressor',
      name: 'Test Effect',
      enabled: true,
      parameters: {},
    };

    it('should render effects processor container', () => {
      render(
        <EffectsProcessor
          effect={testEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByTestId('effects-processor')).toBeInTheDocument();
    });

    it('should render bypass button', () => {
      render(
        <EffectsProcessor
          effect={testEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByTestId('effect-bypass')).toBeInTheDocument();
    });

    it('should render remove button', () => {
      render(
        <EffectsProcessor
          effect={testEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByTestId('remove-effect')).toBeInTheDocument();
    });

    it('should call onToggleBypass when bypass button clicked', () => {
      render(
        <EffectsProcessor
          effect={testEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      const bypassButton = screen.getByTestId('effect-bypass');
      fireEvent.click(bypassButton);

      expect(mockOnToggleBypass).toHaveBeenCalled();
    });

    it('should call onRemove when remove button clicked', () => {
      render(
        <EffectsProcessor
          effect={testEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      const removeButton = screen.getByTestId('remove-effect');
      fireEvent.click(removeButton);

      expect(mockOnRemove).toHaveBeenCalled();
    });

    it('should show bypassed text when effect disabled', () => {
      const disabledEffect = { ...testEffect, enabled: false };

      render(
        <EffectsProcessor
          effect={disabledEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByText('Bypassed')).toBeInTheDocument();
    });

    it('should show bypass text when effect enabled', () => {
      render(
        <EffectsProcessor
          effect={testEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      expect(screen.getByText('Bypass')).toBeInTheDocument();
    });

    it('should apply active class to bypass button when disabled', () => {
      const disabledEffect = { ...testEffect, enabled: false };

      render(
        <EffectsProcessor
          effect={disabledEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      const bypassButton = screen.getByTestId('effect-bypass');
      expect(bypassButton).toHaveClass('active');
    });

    it('should update parameter using number input', () => {
      const eqEffect = {
        id: 'eq-1',
        type: 'eq',
        name: 'EQ',
        enabled: true,
        parameters: { band0Freq: 100 },
      };

      render(
        <EffectsProcessor
          effect={eqEffect}
          onUpdateParameter={mockOnUpdateParameter}
          onToggleBypass={mockOnToggleBypass}
          onRemove={mockOnRemove}
        />
      );

      const numberInput = screen.getByTestId('eq-band-0-freq-input');
      fireEvent.change(numberInput, { target: { value: '500' } });

      expect(mockOnUpdateParameter).toHaveBeenCalledWith('eq-band-0-freq', 500);
    });
  });
});
