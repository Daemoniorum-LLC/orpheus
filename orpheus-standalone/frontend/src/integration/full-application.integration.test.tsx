import { render, screen, fireEvent, waitFor, within } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import App from '../App';

/**
 * Integration Tests: Full Application Workflow
 * Tests complete end-to-end production workflows across all features
 */

describe('Full Application Integration', () => {
  let mockFetch: jest.Mock;

  beforeEach(() => {
    mockFetch = jest.fn();
    global.fetch = mockFetch;
    global.confirm = jest.fn(() => true);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('Complete Music Production Workflow', () => {
    it('should complete full song creation workflow from start to export', async () => {
      // Mock API responses for complete workflow
      const mockProject = {
        id: 'full-workflow-project',
        title: 'My Epic Song',
        artist: 'Test Artist',
        bpm: 128,
        timeSignature: '4/4',
        key: 'Em',
        tracks: [],
      };

      const tracks = [
        {
          id: 'drums-track',
          name: 'Drums',
          trackNumber: 1,
          volume: 0.85,
          pan: 0,
          muted: false,
          soloed: false,
          instrument: 'drums',
          effects: [],
          audioRegions: [
            {
              id: 'drums-region',
              startTime: 0,
              duration: 16,
              audioUrl: '/audio/drums.wav',
            },
          ],
        },
        {
          id: 'bass-track',
          name: 'Bass',
          trackNumber: 2,
          volume: 0.80,
          pan: 0,
          muted: false,
          soloed: false,
          instrument: 'bass',
          effects: [],
          audioRegions: [
            {
              id: 'bass-region',
              startTime: 0,
              duration: 16,
              audioUrl: '/audio/bass.wav',
            },
          ],
        },
        {
          id: 'guitar-track',
          name: 'Guitar',
          trackNumber: 3,
          volume: 0.75,
          pan: 0.7,
          muted: false,
          soloed: false,
          instrument: 'electric-guitar',
          effects: [],
          audioRegions: [
            {
              id: 'guitar-region',
              startTime: 4,
              duration: 12,
              audioUrl: '/audio/guitar.wav',
            },
          ],
        },
      ];

      mockFetch
        // Initial projects list
        .mockResolvedValueOnce({ ok: true, json: async () => [] })
        // Create project
        .mockResolvedValueOnce({ ok: true, json: async () => mockProject })
        // Load project
        .mockResolvedValueOnce({ ok: true, json: async () => mockProject })
        // Create drums track
        .mockResolvedValueOnce({ ok: true, json: async () => tracks[0] })
        // Create bass track
        .mockResolvedValueOnce({ ok: true, json: async () => tracks[1] })
        // Create guitar track
        .mockResolvedValueOnce({ ok: true, json: async () => tracks[2] })
        // Add EQ to guitar
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({
            id: 'eq-1',
            type: 'eq',
            name: 'EQ',
            enabled: true,
            parameters: {},
          }),
        })
        // Add compressor to drums
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({
            id: 'comp-1',
            type: 'compressor',
            name: 'Compressor',
            enabled: true,
            parameters: {},
          }),
        })
        // Export/bounce
        .mockResolvedValueOnce({
          ok: true,
          blob: async () => new Blob(['audio data'], { type: 'audio/wav' }),
        });

      render(
        <BrowserRouter>
          <App />
        </BrowserRouter>
      );

      // ============================================
      // STEP 1: Create New Project
      // ============================================
      await waitFor(() => {
        expect(screen.getByTestId('project-list')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByTestId('new-project-button'));

      await waitFor(() => {
        expect(screen.getByTestId('new-project-form')).toBeInTheDocument();
      });

      fireEvent.change(screen.getByTestId('project-title-input'), {
        target: { value: 'My Epic Song' },
      });
      fireEvent.change(screen.getByTestId('project-artist-input'), {
        target: { value: 'Test Artist' },
      });
      fireEvent.change(screen.getByTestId('project-bpm-input'), {
        target: { value: '128' },
      });
      fireEvent.change(screen.getByTestId('project-key-input'), {
        target: { value: 'Em' },
      });

      fireEvent.click(screen.getByTestId('create-project-submit'));

      await waitFor(() => {
        expect(screen.getByTestId('workspace')).toBeInTheDocument();
      });

      // ============================================
      // STEP 2: Add Multiple Tracks
      // ============================================

      // Add Drums
      fireEvent.click(screen.getByTestId('add-track-button'));
      await waitFor(() => {
        expect(screen.getByTestId('add-track-dialog')).toBeInTheDocument();
      });
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

      // Add Bass
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

      // Add Guitar
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

      // Verify all tracks created
      const trackItems = screen.getAllByTestId('track-item');
      expect(trackItems).toHaveLength(3);

      // ============================================
      // STEP 3: Mix Tracks
      // ============================================

      const volumeSliders = screen.getAllByTestId('track-volume-slider');
      const panSliders = screen.getAllByTestId('track-pan-slider');

      // Adjust drums volume
      fireEvent.change(volumeSliders[0], { target: { value: '0.85' } });

      // Adjust bass volume
      fireEvent.change(volumeSliders[1], { target: { value: '0.80' } });

      // Adjust guitar volume and pan
      fireEvent.change(volumeSliders[2], { target: { value: '0.75' } });
      fireEvent.change(panSliders[2], { target: { value: '0.7' } });

      // ============================================
      // STEP 4: Add Effects
      // ============================================

      // Open mixer panel
      await waitFor(() => {
        expect(screen.getByTestId('mixer-panel')).toBeInTheDocument();
      });

      // Add EQ to guitar
      const effectsButtons = screen.getAllByTestId('track-effects-button');
      fireEvent.click(effectsButtons[2]); // Guitar track

      await waitFor(() => {
        expect(screen.getByTestId('effects-panel')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByTestId('add-effect-button'));
      fireEvent.click(screen.getByTestId('effect-eq'));

      await waitFor(() => {
        expect(screen.getByText('EQ')).toBeInTheDocument();
      });

      // Close effects panel
      fireEvent.click(screen.getAllByText('×')[0]);

      // Add compressor to drums
      fireEvent.click(effectsButtons[0]); // Drums track

      await waitFor(() => {
        expect(screen.getByTestId('effects-panel')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByTestId('add-effect-button'));
      fireEvent.click(screen.getByTestId('effect-compressor'));

      await waitFor(() => {
        expect(screen.getByText('Compressor')).toBeInTheDocument();
      });

      // ============================================
      // STEP 5: Playback Test
      // ============================================

      // Set loop points
      fireEvent.click(screen.getByTestId('loop-button'));

      // Start playback
      const playButton = screen.getByTestId('transport-play-button');
      fireEvent.click(playButton);

      await waitFor(() => {
        expect(playButton).toHaveClass('playing');
      });

      // Let it play briefly
      await new Promise(resolve => setTimeout(resolve, 100));

      // Stop playback
      fireEvent.click(screen.getByTestId('transport-stop-button'));

      await waitFor(() => {
        expect(playButton).not.toHaveClass('playing');
      });

      // ============================================
      // STEP 6: Export Final Mix
      // ============================================

      // Select all tracks
      fireEvent.click(screen.getByTestId('select-all-button'));

      // Open bounce dialog
      fireEvent.click(screen.getByTestId('bounce-audio-button'));

      await waitFor(() => {
        expect(screen.getByTestId('bounce-format-wav')).toBeInTheDocument();
      });

      // Configure export
      fireEvent.click(screen.getByTestId('bounce-format-wav'));
      fireEvent.change(screen.getByTestId('bounce-sample-rate'), {
        target: { value: '48000' },
      });
      fireEvent.change(screen.getByTestId('bounce-bit-depth'), {
        target: { value: '24' },
      });

      // Start export
      fireEvent.click(screen.getByTestId('bounce-confirm'));

      await waitFor(() => {
        expect(screen.getByTestId('render-progress')).toBeInTheDocument();
      });

      // Verify export completed
      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          expect.stringContaining('/export'),
          expect.objectContaining({
            method: 'POST',
          })
        );
      });
    });
  });

  describe('Collaborative Production Workflow', () => {
    it('should handle multi-user production session', async () => {
      const mockProject = {
        id: 'collab-project',
        title: 'Collaborative Production',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [],
      };

      const mockCollaborators = [
        {
          id: 'user-1',
          email: 'producer@example.com',
          name: 'Producer',
          role: 'owner',
          active: true,
          avatarColor: '#3b82f6',
        },
        {
          id: 'user-2',
          email: 'engineer@example.com',
          name: 'Engineer',
          role: 'editor',
          active: true,
          avatarColor: '#10b981',
        },
      ];

      mockFetch
        .mockResolvedValueOnce({ ok: true, json: async () => [mockProject] })
        .mockResolvedValueOnce({ ok: true, json: async () => mockProject })
        .mockResolvedValueOnce({ ok: true, json: async () => mockCollaborators });

      render(
        <BrowserRouter>
          <App />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByTestId('project-list')).toBeInTheDocument();
      });

      // Open project
      fireEvent.click(screen.getByTestId('project-card'));

      await waitFor(() => {
        expect(screen.getByTestId('workspace')).toBeInTheDocument();
      });

      // Verify collaboration toolbar
      await waitFor(() => {
        expect(screen.getByTestId('collaboration-toolbar')).toBeInTheDocument();
      });

      // Should show active collaborators
      expect(screen.getByText(/2 active/)).toBeInTheDocument();

      // Verify online status
      expect(screen.getByTestId('online-indicator')).toBeInTheDocument();
    });
  });

  describe('Error Recovery Workflow', () => {
    it('should recover from network errors gracefully', async () => {
      const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation();

      mockFetch
        // Initial load fails
        .mockRejectedValueOnce(new Error('Network error'))
        // Retry succeeds
        .mockResolvedValueOnce({
          ok: true,
          json: async () => [{
            id: 'project-1',
            title: 'Recovered Project',
            bpm: 120,
            timeSignature: '4/4',
          }],
        });

      render(
        <BrowserRouter>
          <App />
        </BrowserRouter>
      );

      // Should handle error
      await waitFor(() => {
        expect(consoleErrorSpy).toHaveBeenCalled();
      });

      // Retry loading
      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledTimes(2);
      });

      consoleErrorSpy.mockRestore();
    });

    it('should handle auto-save failures and retry', async () => {
      const mockProject = {
        id: 'autosave-project',
        title: 'Auto-save Test',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [
          {
            id: 'track-1',
            name: 'Track',
            volume: 0.8,
            pan: 0,
          },
        ],
      };

      mockFetch
        .mockResolvedValueOnce({ ok: true, json: async () => [mockProject] })
        .mockResolvedValueOnce({ ok: true, json: async () => mockProject })
        .mockRejectedValueOnce(new Error('Save failed'))
        .mockResolvedValueOnce({ ok: true, json: async () => ({}) });

      render(
        <BrowserRouter>
          <App />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByTestId('project-list')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByTestId('project-card'));

      await waitFor(() => {
        expect(screen.getByTestId('workspace')).toBeInTheDocument();
      });

      // Make change
      const volumeSlider = screen.getByTestId('track-volume-slider');
      fireEvent.change(volumeSlider, { target: { value: '0.5' } });

      // Auto-save indicator should eventually show saved
      await waitFor(() => {
        const indicator = screen.getByTestId('auto-save-indicator');
        expect(indicator.textContent).toMatch(/Saved|Saving/);
      }, { timeout: 6000 });
    });
  });

  describe('Performance Under Load', () => {
    it('should handle large project with many tracks', async () => {
      const largeTracks = Array.from({ length: 20 }, (_, i) => ({
        id: `track-${i}`,
        name: `Track ${i + 1}`,
        trackNumber: i + 1,
        volume: 0.8,
        pan: 0,
        muted: false,
        soloed: false,
        audioRegions: [],
      }));

      const mockProject = {
        id: 'large-project',
        title: 'Large Project',
        bpm: 120,
        timeSignature: '4/4',
        tracks: largeTracks,
      };

      mockFetch
        .mockResolvedValueOnce({ ok: true, json: async () => [mockProject] })
        .mockResolvedValueOnce({ ok: true, json: async () => mockProject });

      render(
        <BrowserRouter>
          <App />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByText('Large Project')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByTestId('project-card'));

      await waitFor(() => {
        expect(screen.getByTestId('workspace')).toBeInTheDocument();
      });

      // Should render all tracks
      const trackItems = screen.getAllByTestId('track-item');
      expect(trackItems).toHaveLength(20);

      // UI should remain responsive
      const startTime = performance.now();
      fireEvent.click(screen.getByTestId('zoom-in-button'));
      const endTime = performance.now();

      expect(endTime - startTime).toBeLessThan(1000);
    });
  });

  describe('Data Persistence', () => {
    it('should persist project state across page refresh', async () => {
      const mockProject = {
        id: 'persist-project',
        title: 'Persistence Test',
        bpm: 140,
        timeSignature: '7/8',
        tracks: [
          {
            id: 'track-1',
            name: 'Important Track',
            volume: 0.9,
            pan: 0.5,
          },
        ],
      };

      mockFetch
        .mockResolvedValueOnce({ ok: true, json: async () => [mockProject] })
        .mockResolvedValueOnce({ ok: true, json: async () => mockProject });

      const { unmount } = render(
        <BrowserRouter>
          <App />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByText('Persistence Test')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByTestId('project-card'));

      await waitFor(() => {
        expect(screen.getByText('Important Track')).toBeInTheDocument();
      });

      const projectId = screen.getByTestId('project-id').getAttribute('data-id');

      // Simulate page refresh
      unmount();

      mockFetch.mockResolvedValueOnce({ ok: true, json: async () => mockProject });

      render(
        <BrowserRouter>
          <App />
        </BrowserRouter>
      );

      // Should reload same project
      await waitFor(() => {
        const newProjectId = screen.getByTestId('project-id').getAttribute('data-id');
        expect(newProjectId).toBe(projectId);
      });

      // Track should still exist
      expect(screen.getByText('Important Track')).toBeInTheDocument();
    });
  });
});
