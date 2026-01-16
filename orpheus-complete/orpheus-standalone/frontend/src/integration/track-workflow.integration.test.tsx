import { render, screen, fireEvent, waitFor, within } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { Workspace } from '../components/Workspace';

/**
 * Integration Tests: Track Management Workflow
 * Tests complete track creation, editing, and playback workflows
 */

describe('Track Management Integration', () => {
  let mockFetch: jest.Mock;

  beforeEach(() => {
    mockFetch = jest.fn();
    global.fetch = mockFetch;
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('Track Creation and Configuration Flow', () => {
    it('should create track and configure all parameters', async () => {
      const mockProject = {
        id: 'test-project',
        title: 'Test Project',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [],
      };

      const newTrack = {
        id: 'track-1',
        name: 'Drums',
        trackNumber: 1,
        volume: 0.8,
        pan: 0,
        muted: false,
        soloed: false,
        recordArmed: false,
        peakLevel: 0,
        rmsLevel: 0,
        instrument: 'drums',
      };

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProject,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => newTrack,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ ...newTrack, volume: 0.5 }),
        });

      render(
        <BrowserRouter>
          <Workspace />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByTestId('workspace')).toBeInTheDocument();
      });

      // Open add track dialog
      const addTrackButton = screen.getByTestId('add-track-button');
      fireEvent.click(addTrackButton);

      await waitFor(() => {
        expect(screen.getByTestId('add-track-dialog')).toBeInTheDocument();
      });

      // Fill track details
      fireEvent.change(screen.getByTestId('track-name-input'), {
        target: { value: 'Drums' },
      });
      fireEvent.change(screen.getByTestId('track-instrument-select'), {
        target: { value: 'drums' },
      });

      // Create track
      fireEvent.click(screen.getByTestId('create-track-submit'));

      // Should call API to create track
      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          expect.stringContaining('/tracks'),
          expect.objectContaining({
            method: 'POST',
            body: JSON.stringify({
              name: 'Drums',
              instrument: 'drums',
            }),
          })
        );
      });

      // Track should appear in list
      await waitFor(() => {
        expect(screen.getByText('Drums')).toBeInTheDocument();
      });

      // Adjust volume
      const volumeSlider = screen.getByTestId('track-volume-slider');
      fireEvent.change(volumeSlider, { target: { value: '0.5' } });

      // Should update track via API
      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          expect.stringContaining('/tracks/track-1'),
          expect.objectContaining({
            method: 'PATCH',
          })
        );
      });
    });

    it('should create multiple tracks and manage them', async () => {
      const mockProject = {
        id: 'test-project',
        title: 'Multi Track Test',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [],
      };

      const tracks = [
        {
          id: 'track-1',
          name: 'Drums',
          trackNumber: 1,
          volume: 0.8,
          pan: 0,
          muted: false,
          soloed: false,
          instrument: 'drums',
        },
        {
          id: 'track-2',
          name: 'Bass',
          trackNumber: 2,
          volume: 0.7,
          pan: 0,
          muted: false,
          soloed: false,
          instrument: 'bass',
        },
        {
          id: 'track-3',
          name: 'Guitar',
          trackNumber: 3,
          volume: 0.75,
          pan: 0.5,
          muted: false,
          soloed: false,
          instrument: 'electric-guitar',
        },
      ];

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProject,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => tracks[0],
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => tracks[1],
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => tracks[2],
        });

      render(
        <BrowserRouter>
          <Workspace />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByTestId('workspace')).toBeInTheDocument();
      });

      // Create first track
      fireEvent.click(screen.getByTestId('add-track-button'));
      fireEvent.change(screen.getByTestId('track-name-input'), {
        target: { value: 'Drums' },
      });
      fireEvent.change(screen.getByTestId('track-instrument-select'), {
        target: { value: 'drums' },
      });
      fireEvent.click(screen.getByTestId('create-track-submit'));

      await waitFor(() => {
        expect(screen.getByText('Drums')).toBeInTheDocument();
      });

      // Create second track
      fireEvent.click(screen.getByTestId('add-track-button'));
      fireEvent.change(screen.getByTestId('track-name-input'), {
        target: { value: 'Bass' },
      });
      fireEvent.change(screen.getByTestId('track-instrument-select'), {
        target: { value: 'bass' },
      });
      fireEvent.click(screen.getByTestId('create-track-submit'));

      await waitFor(() => {
        expect(screen.getByText('Bass')).toBeInTheDocument();
      });

      // Create third track
      fireEvent.click(screen.getByTestId('add-track-button'));
      fireEvent.change(screen.getByTestId('track-name-input'), {
        target: { value: 'Guitar' },
      });
      fireEvent.change(screen.getByTestId('track-instrument-select'), {
        target: { value: 'electric-guitar' },
      });
      fireEvent.click(screen.getByTestId('create-track-submit'));

      await waitFor(() => {
        expect(screen.getByText('Guitar')).toBeInTheDocument();
      });

      // All tracks should be visible
      const trackItems = screen.getAllByTestId('track-item');
      expect(trackItems).toHaveLength(3);
    });
  });

  describe('Track Mute/Solo Integration', () => {
    it('should handle mute/solo interactions across tracks', async () => {
      const mockProject = {
        id: 'test-project',
        title: 'Mute Solo Test',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [
          {
            id: 'track-1',
            name: 'Track 1',
            volume: 0.8,
            pan: 0,
            muted: false,
            soloed: false,
          },
          {
            id: 'track-2',
            name: 'Track 2',
            volume: 0.8,
            pan: 0,
            muted: false,
            soloed: false,
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
          json: async () => ({ ...mockProject.tracks[0], muted: true }),
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ ...mockProject.tracks[1], soloed: true }),
        });

      render(
        <BrowserRouter>
          <Workspace />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByText('Track 1')).toBeInTheDocument();
        expect(screen.getByText('Track 2')).toBeInTheDocument();
      });

      const trackItems = screen.getAllByTestId('track-item');

      // Mute first track
      const muteButton1 = within(trackItems[0]).getByTestId('track-mute-button');
      fireEvent.click(muteButton1);

      await waitFor(() => {
        expect(muteButton1).toHaveClass('active');
      });

      // Solo second track
      const soloButton2 = within(trackItems[1]).getByTestId('track-solo-button');
      fireEvent.click(soloButton2);

      await waitFor(() => {
        expect(soloButton2).toHaveClass('active');
      });
    });
  });

  describe('Track Volume and Pan Integration', () => {
    it('should adjust multiple track volumes and pan simultaneously', async () => {
      const mockProject = {
        id: 'test-project',
        title: 'Mixing Test',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [
          {
            id: 'track-1',
            name: 'Drums',
            volume: 0.8,
            pan: 0,
            muted: false,
            soloed: false,
          },
          {
            id: 'track-2',
            name: 'Bass',
            volume: 0.8,
            pan: 0,
            muted: false,
            soloed: false,
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
        expect(screen.getByText('Drums')).toBeInTheDocument();
      });

      const volumeSliders = screen.getAllByTestId('track-volume-slider');
      const panSliders = screen.getAllByTestId('track-pan-slider');

      // Adjust drums volume
      fireEvent.change(volumeSliders[0], { target: { value: '0.9' } });

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          expect.stringContaining('/tracks/track-1'),
          expect.objectContaining({
            method: 'PATCH',
          })
        );
      });

      // Adjust bass pan to right
      fireEvent.change(panSliders[1], { target: { value: '0.5' } });

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          expect.stringContaining('/tracks/track-2'),
          expect.objectContaining({
            method: 'PATCH',
          })
        );
      });
    });
  });

  describe('Track Recording Workflow', () => {
    it('should arm track, record, and create audio region', async () => {
      const mockProject = {
        id: 'test-project',
        title: 'Recording Test',
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
            recordArmed: false,
            audioRegions: [],
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
          json: async () => ({ ...mockProject.tracks[0], recordArmed: true }),
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({
            id: 'region-1',
            startTime: 0,
            duration: 5,
            audioUrl: '/audio/recording.wav',
          }),
        });

      render(
        <BrowserRouter>
          <Workspace />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByText('Guitar')).toBeInTheDocument();
      });

      // Arm track for recording
      const recordArmButton = screen.getByTestId('track-record-arm');
      fireEvent.click(recordArmButton);

      await waitFor(() => {
        expect(recordArmButton).toHaveClass('active');
      });

      // Start recording
      const recordButton = screen.getByTestId('transport-record-button');
      fireEvent.click(recordButton);

      // Start playback
      const playButton = screen.getByTestId('transport-play-button');
      fireEvent.click(playButton);

      // Simulate recording time
      await new Promise(resolve => setTimeout(resolve, 100));

      // Stop recording
      const stopButton = screen.getByTestId('transport-stop-button');
      fireEvent.click(stopButton);

      // Audio region should be created
      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          expect.stringContaining('/regions'),
          expect.objectContaining({
            method: 'POST',
          })
        );
      });
    });
  });

  describe('Track and Timeline Integration', () => {
    it('should display tracks in timeline with audio regions', async () => {
      const mockProject = {
        id: 'test-project',
        title: 'Timeline Test',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [
          {
            id: 'track-1',
            name: 'Drums',
            trackNumber: 1,
            volume: 0.8,
            pan: 0,
            audioRegions: [
              {
                id: 'region-1',
                startTime: 0,
                duration: 5,
                audioUrl: '/audio/drums.wav',
              },
              {
                id: 'region-2',
                startTime: 10,
                duration: 3,
                audioUrl: '/audio/drums2.wav',
              },
            ],
          },
          {
            id: 'track-2',
            name: 'Bass',
            trackNumber: 2,
            volume: 0.8,
            pan: 0,
            audioRegions: [
              {
                id: 'region-3',
                startTime: 0,
                duration: 8,
                audioUrl: '/audio/bass.wav',
              },
            ],
          },
        ],
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockProject,
      });

      render(
        <BrowserRouter>
          <Workspace />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByTestId('timeline')).toBeInTheDocument();
      });

      // Timeline should show both tracks
      const timelineTracks = screen.getAllByClassName('timeline-track');
      expect(timelineTracks).toHaveLength(2);

      // Should show all audio regions
      const audioRegions = screen.getAllByTestId('audio-region');
      expect(audioRegions).toHaveLength(3);
    });
  });

  describe('Track and Mixer Integration', () => {
    it('should sync track controls between track list and mixer panel', async () => {
      const mockProject = {
        id: 'test-project',
        title: 'Mixer Sync Test',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [
          {
            id: 'track-1',
            name: 'Synth',
            volume: 0.8,
            pan: 0,
            muted: false,
            soloed: false,
            peakLevel: 0.5,
            rmsLevel: 0.3,
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

      // Get volume slider from track list
      const trackVolumeSlider = within(screen.getByTestId('track-list'))
        .getByTestId('track-volume-slider');

      // Change volume in track list
      fireEvent.change(trackVolumeSlider, { target: { value: '0.6' } });

      // Mixer panel should reflect the change
      await waitFor(() => {
        const mixerVolumeSlider = within(screen.getByTestId('mixer-panel'))
          .getByTestId('track-volume-slider');
        expect((mixerVolumeSlider as HTMLInputElement).value).toBe('0.6');
      });
    });
  });
});
