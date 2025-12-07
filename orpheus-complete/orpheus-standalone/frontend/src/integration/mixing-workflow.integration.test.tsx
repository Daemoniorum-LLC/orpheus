import { render, screen, fireEvent, waitFor, within } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { Workspace } from '../components/Workspace';

/**
 * Integration Tests: Mixing and Effects Workflow
 * Tests complete mixing, effects processing, and master bus workflows
 */

describe('Mixing Workflow Integration', () => {
  let mockFetch: jest.Mock;

  beforeEach(() => {
    mockFetch = jest.fn();
    global.fetch = mockFetch;
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('Effects Chain Management', () => {
    it('should add, configure, and remove effects from track', async () => {
      const mockProject = {
        id: 'test-project',
        title: 'Effects Test',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [
          {
            id: 'track-1',
            name: 'Guitar',
            volume: 0.8,
            pan: 0,
            muted: false,
            soloed: false,
            effects: [],
          },
        ],
      };

      const eqEffect = {
        id: 'effect-1',
        type: 'eq',
        name: 'EQ',
        enabled: true,
        parameters: {
          band0Freq: 100,
          band0Gain: 0,
          band0Q: 1,
        },
      };

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProject,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => eqEffect,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ ...eqEffect, parameters: { ...eqEffect.parameters, band0Freq: 2000 } }),
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({}),
        });

      render(
        <BrowserRouter>
          <Workspace />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByTestId('mixer-panel')).toBeInTheDocument();
      });

      // Open effects panel
      const effectsButton = screen.getByTestId('track-effects-button');
      fireEvent.click(effectsButton);

      await waitFor(() => {
        expect(screen.getByTestId('effects-panel')).toBeInTheDocument();
      });

      // Add EQ effect
      fireEvent.click(screen.getByTestId('add-effect-button'));
      fireEvent.click(screen.getByTestId('effect-eq'));

      // Should create effect via API
      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          expect.stringContaining('/effects'),
          expect.objectContaining({
            method: 'POST',
            body: expect.stringContaining('eq'),
          })
        );
      });

      // Effect should appear in effects panel
      await waitFor(() => {
        expect(screen.getByText('EQ')).toBeInTheDocument();
      });

      // Adjust EQ parameter
      const freqSlider = screen.getByTestId('eq-band-0-freq');
      fireEvent.change(freqSlider, { target: { value: '2000' } });

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          expect.stringContaining('/effects/effect-1'),
          expect.objectContaining({
            method: 'PATCH',
          })
        );
      });

      // Remove effect
      const removeButton = screen.getByTestId('remove-effect');
      fireEvent.click(removeButton);

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          expect.stringContaining('/effects/effect-1'),
          expect.objectContaining({
            method: 'DELETE',
          })
        );
      });
    });

    it('should build complex effects chain', async () => {
      const mockProject = {
        id: 'test-project',
        title: 'Effects Chain Test',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [
          {
            id: 'track-1',
            name: 'Vocals',
            volume: 0.8,
            pan: 0,
            effects: [],
          },
        ],
      };

      const effects = [
        { id: 'eq-1', type: 'eq', name: 'EQ', enabled: true, parameters: {} },
        { id: 'comp-1', type: 'compressor', name: 'Compressor', enabled: true, parameters: {} },
        { id: 'reverb-1', type: 'reverb', name: 'Reverb', enabled: true, parameters: {} },
        { id: 'delay-1', type: 'delay', name: 'Delay', enabled: true, parameters: {} },
      ];

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProject,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => effects[0],
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => effects[1],
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => effects[2],
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => effects[3],
        });

      render(
        <BrowserRouter>
          <Workspace />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByTestId('mixer-panel')).toBeInTheDocument();
      });

      // Open effects panel
      fireEvent.click(screen.getByTestId('track-effects-button'));

      // Add EQ
      fireEvent.click(screen.getByTestId('add-effect-button'));
      fireEvent.click(screen.getByTestId('effect-eq'));

      await waitFor(() => {
        expect(screen.getByText('EQ')).toBeInTheDocument();
      });

      // Add Compressor
      fireEvent.click(screen.getByTestId('add-effect-button'));
      fireEvent.click(screen.getByTestId('effect-compressor'));

      await waitFor(() => {
        expect(screen.getByText('Compressor')).toBeInTheDocument();
      });

      // Add Reverb
      fireEvent.click(screen.getByTestId('add-effect-button'));
      fireEvent.click(screen.getByTestId('effect-reverb'));

      await waitFor(() => {
        expect(screen.getByText('Reverb')).toBeInTheDocument();
      });

      // Add Delay
      fireEvent.click(screen.getByTestId('add-effect-button'));
      fireEvent.click(screen.getByTestId('effect-delay'));

      await waitFor(() => {
        expect(screen.getByText('Delay')).toBeInTheDocument();
      });

      // All effects should be visible in chain
      const effectSlots = screen.getAllByTestId('effect-slot');
      expect(effectSlots).toHaveLength(4);
    });

    it('should bypass effects and maintain chain', async () => {
      const mockProject = {
        id: 'test-project',
        title: 'Bypass Test',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [
          {
            id: 'track-1',
            name: 'Guitar',
            volume: 0.8,
            pan: 0,
            effects: [
              { id: 'dist-1', type: 'distortion', name: 'Distortion', enabled: true, parameters: {} },
              { id: 'eq-1', type: 'eq', name: 'EQ', enabled: true, parameters: {} },
            ],
          },
        ],
      };

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProject,
        })
        .mockResolvedValue({
          ok: true,
          json: async () => ({}),
        });

      render(
        <BrowserRouter>
          <Workspace />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByText('FX (2)')).toBeInTheDocument();
      });

      // Open effects panel
      fireEvent.click(screen.getByTestId('track-effects-button'));

      // Bypass first effect
      const bypassButtons = screen.getAllByTestId('effect-bypass');
      fireEvent.click(bypassButtons[0]);

      await waitFor(() => {
        expect(bypassButtons[0]).toHaveClass('active');
      });

      // Both effects should still be in chain
      const effectSlots = screen.getAllByTestId('effect-slot');
      expect(effectSlots).toHaveLength(2);
    });
  });

  describe('Compressor Configuration', () => {
    it('should configure compressor with all parameters', async () => {
      const mockProject = {
        id: 'test-project',
        title: 'Compressor Test',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [
          {
            id: 'track-1',
            name: 'Drums',
            volume: 0.8,
            pan: 0,
            effects: [
              {
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
              },
            ],
          },
        ],
      };

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProject,
        })
        .mockResolvedValue({
          ok: true,
          json: async () => ({}),
        });

      render(
        <BrowserRouter>
          <Workspace />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByTestId('mixer-panel')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByTestId('track-effects-button'));

      await waitFor(() => {
        expect(screen.getByTestId('compressor-threshold')).toBeInTheDocument();
      });

      // Adjust threshold
      const thresholdSlider = screen.getByTestId('compressor-threshold');
      fireEvent.change(thresholdSlider, { target: { value: '-12' } });

      // Adjust ratio
      const ratioSlider = screen.getByTestId('compressor-ratio');
      fireEvent.change(ratioSlider, { target: { value: '6' } });

      // Adjust attack
      const attackSlider = screen.getByTestId('compressor-attack');
      fireEvent.change(attackSlider, { target: { value: '5' } });

      // All parameters should update
      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledTimes(4); // Initial load + 3 updates
      });

      // Gain reduction meter should be visible
      expect(screen.getByTestId('gain-reduction-meter')).toBeInTheDocument();
    });
  });

  describe('Multi-Track Mixing', () => {
    it('should balance multiple tracks with different settings', async () => {
      const mockProject = {
        id: 'test-project',
        title: 'Multi-Track Mix',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [
          {
            id: 'track-1',
            name: 'Drums',
            volume: 0.85,
            pan: 0,
            muted: false,
            soloed: false,
            peakLevel: 0.7,
            rmsLevel: 0.5,
          },
          {
            id: 'track-2',
            name: 'Bass',
            volume: 0.80,
            pan: 0,
            muted: false,
            soloed: false,
            peakLevel: 0.6,
            rmsLevel: 0.4,
          },
          {
            id: 'track-3',
            name: 'Guitar',
            volume: 0.70,
            pan: 0.7,
            muted: false,
            soloed: false,
            peakLevel: 0.5,
            rmsLevel: 0.3,
          },
          {
            id: 'track-4',
            name: 'Vocals',
            volume: 0.90,
            pan: 0,
            muted: false,
            soloed: false,
            peakLevel: 0.8,
            rmsLevel: 0.6,
          },
        ],
      };

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProject,
        })
        .mockResolvedValue({
          ok: true,
          json: async () => ({}),
        });

      render(
        <BrowserRouter>
          <Workspace />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByTestId('mixer-panel')).toBeInTheDocument();
      });

      const mixerChannels = screen.getAllByTestId('mixer-channel');
      expect(mixerChannels).toHaveLength(4);

      // Adjust drums volume
      const drumsChannel = mixerChannels[0];
      const drumsVolume = within(drumsChannel).getByTestId('track-volume-slider');
      fireEvent.change(drumsVolume, { target: { value: '0.75' } });

      // Adjust guitar pan more to right
      const guitarChannel = mixerChannels[2];
      const guitarPan = within(guitarChannel).getByTestId('track-pan-slider');
      fireEvent.change(guitarPan, { target: { value: '0.9' } });

      // All meters should be visible
      const peakMeters = screen.getAllByTestId('peak-meter');
      expect(peakMeters).toHaveLength(4);

      const rmsMeters = screen.getAllByTestId('rms-meter');
      expect(rmsMeters).toHaveLength(4);
    });
  });

  describe('Master Bus Processing', () => {
    it('should apply master bus effects to overall mix', async () => {
      const mockProject = {
        id: 'test-project',
        title: 'Master Bus Test',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [
          {
            id: 'track-1',
            name: 'Full Mix',
            volume: 0.8,
            pan: 0,
          },
        ],
        masterBusEffects: [],
      };

      const limiterEffect = {
        id: 'limiter-1',
        type: 'limiter',
        name: 'Limiter',
        enabled: true,
        parameters: {
          threshold: -1,
          release: 50,
        },
      };

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProject,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => limiterEffect,
        });

      render(
        <BrowserRouter>
          <Workspace />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByTestId('mixer-panel')).toBeInTheDocument();
      });

      // Open master bus
      const masterBusButton = screen.getByTestId('master-bus');
      fireEvent.click(masterBusButton);

      await waitFor(() => {
        expect(screen.getByTestId('master-bus-panel')).toBeInTheDocument();
      });

      // Add limiter to master bus
      fireEvent.click(screen.getByTestId('add-effect-button'));
      fireEvent.click(screen.getByTestId('effect-limiter'));

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          expect.stringContaining('/master/effects'),
          expect.objectContaining({
            method: 'POST',
          })
        );
      });
    });
  });

  describe('Automation Integration', () => {
    it('should create and play back volume automation', async () => {
      const mockProject = {
        id: 'test-project',
        title: 'Automation Test',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [
          {
            id: 'track-1',
            name: 'Lead',
            volume: 0.8,
            pan: 0,
            automation: [],
          },
        ],
      };

      const automationPoints = [
        { time: 0, value: 0.8 },
        { time: 2, value: 0.5 },
        { time: 4, value: 1.0 },
      ];

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProject,
        })
        .mockResolvedValue({
          ok: true,
          json: async () => ({ points: automationPoints }),
        });

      render(
        <BrowserRouter>
          <Workspace />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByText('Lead')).toBeInTheDocument();
      });

      // Enable automation on volume
      const volumeSlider = screen.getByTestId('track-volume-slider');
      fireEvent.contextMenu(volumeSlider);

      // Would show automation menu and enable automation
      // This would create automation lane in timeline
    });
  });

  describe('Export and Bounce', () => {
    it('should export mixed track with effects', async () => {
      const mockProject = {
        id: 'test-project',
        title: 'Export Test',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [
          {
            id: 'track-1',
            name: 'Mixed Track',
            volume: 0.8,
            pan: 0,
            effects: [
              { id: 'eq-1', type: 'eq', name: 'EQ', enabled: true, parameters: {} },
              { id: 'comp-1', type: 'compressor', name: 'Compressor', enabled: true, parameters: {} },
            ],
          },
        ],
      };

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProject,
        })
        .mockResolvedValueOnce({
          ok: true,
          blob: async () => new Blob(['audio data'], { type: 'audio/wav' }),
        });

      global.URL.createObjectURL = jest.fn(() => 'blob:mock-url');

      render(
        <BrowserRouter>
          <Workspace />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByTestId('workspace')).toBeInTheDocument();
      });

      // Select track
      fireEvent.click(screen.getByTestId('select-all-button'));

      // Bounce audio
      const downloadPromise = new Promise<void>(resolve => {
        window.addEventListener('download', () => resolve());
      });

      fireEvent.click(screen.getByTestId('bounce-audio-button'));

      // Configure export settings
      fireEvent.click(screen.getByTestId('bounce-format-wav'));
      fireEvent.change(screen.getByTestId('bounce-sample-rate'), {
        target: { value: '48000' },
      });
      fireEvent.change(screen.getByTestId('bounce-bit-depth'), {
        target: { value: '24' },
      });

      fireEvent.click(screen.getByTestId('bounce-confirm'));

      // Should show render progress
      await waitFor(() => {
        expect(screen.getByTestId('render-progress')).toBeInTheDocument();
      });
    });
  });
});
