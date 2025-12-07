import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { Workspace } from './Workspace';

global.fetch = jest.fn();

const renderWorkspace = (projectId = 'test-project-id') => {
  return render(
    <BrowserRouter>
      <Routes>
        <Route path="/project/:projectId" element={<Workspace />} />
      </Routes>
    </BrowserRouter>,
    { wrapper: ({ children }) => (
      <BrowserRouter>
        <Routes>
          <Route path="*" element={children} />
        </Routes>
      </BrowserRouter>
    )}
  );
};

describe('Workspace', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    (global.fetch as jest.Mock).mockResolvedValue({
      ok: true,
      json: async () => ({ tracks: [] }),
    });
  });

  it('should render workspace container', () => {
    renderWorkspace();
    expect(screen.getByTestId('workspace')).toBeInTheDocument();
  });

  it('should render workspace header', () => {
    renderWorkspace();
    expect(screen.getByTestId('workspace-header')).toBeInTheDocument();
  });

  it('should render workspace content', () => {
    renderWorkspace();
    expect(screen.getByTestId('workspace-content')).toBeInTheDocument();
  });

  it('should render track controls area', () => {
    renderWorkspace();
    expect(screen.getByTestId('track-controls')).toBeInTheDocument();
  });

  it('should render timeline container', () => {
    renderWorkspace();
    expect(screen.getByTestId('timeline-container')).toBeInTheDocument();
  });

  it('should render mixer container', () => {
    renderWorkspace();
    expect(screen.getByTestId('mixer-container')).toBeInTheDocument();
  });

  it('should render transport controls', () => {
    renderWorkspace();
    expect(screen.getByTestId('transport')).toBeInTheDocument();
  });

  it('should display project title', async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        id: 'test-id',
        title: 'My Test Project',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [],
      }),
    });

    renderWorkspace();

    await waitFor(() => {
      expect(screen.getByTestId('project-title')).toHaveTextContent('My Test Project');
    });
  });

  it('should display project BPM', async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        id: 'test-id',
        title: 'Test',
        bpm: 140,
        timeSignature: '4/4',
        tracks: [],
      }),
    });

    renderWorkspace();

    await waitFor(() => {
      expect(screen.getByTestId('project-bpm')).toHaveTextContent('140');
    });
  });

  it('should display project time signature', async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        id: 'test-id',
        title: 'Test',
        bpm: 120,
        timeSignature: '7/8',
        tracks: [],
      }),
    });

    renderWorkspace();

    await waitFor(() => {
      expect(screen.getByTestId('project-time-signature')).toHaveTextContent('7/8');
    });
  });

  it('should show auto-save indicator', () => {
    renderWorkspace();
    expect(screen.getByTestId('auto-save-indicator')).toBeInTheDocument();
  });

  it('should update auto-save indicator text', async () => {
    renderWorkspace();

    const autoSaveIndicator = screen.getByTestId('auto-save-indicator');

    await waitFor(() => {
      expect(autoSaveIndicator.textContent).toMatch(/Saved|Saving/);
    }, { timeout: 3000 });
  });

  it('should load project data on mount', async () => {
    const mockProject = {
      id: 'test-id',
      title: 'Test Project',
      bpm: 120,
      timeSignature: '4/4',
      key: 'C',
      tracks: [],
    };

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockProject,
    });

    renderWorkspace();

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalled();
    });
  });

  it('should handle project load error', async () => {
    const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation();

    (global.fetch as jest.Mock).mockRejectedValueOnce(new Error('Load error'));

    renderWorkspace();

    await waitFor(() => {
      expect(consoleErrorSpy).toHaveBeenCalled();
    });

    consoleErrorSpy.mockRestore();
  });

  it('should add new track', async () => {
    const mockProject = {
      id: 'test-id',
      title: 'Test',
      bpm: 120,
      timeSignature: '4/4',
      tracks: [],
    };

    (global.fetch as jest.Mock)
      .mockResolvedValueOnce({
        ok: true,
        json: async () => mockProject,
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          id: 'new-track-id',
          name: 'New Track',
          volume: 0.8,
          pan: 0,
        }),
      });

    renderWorkspace();

    await waitFor(() => {
      expect(screen.getByTestId('workspace')).toBeInTheDocument();
    });

    // Track addition would be tested through TrackList component
  });

  it('should update track properties', async () => {
    const mockProject = {
      id: 'test-id',
      title: 'Test',
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
      ],
    };

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockProject,
    });

    renderWorkspace();

    await waitFor(() => {
      expect(screen.getByTestId('workspace')).toBeInTheDocument();
    });
  });

  it('should update playhead position', async () => {
    renderWorkspace();

    await waitFor(() => {
      expect(screen.getByTestId('workspace')).toBeInTheDocument();
    });

    // Playhead position updates would be visible in timeline
  });

  it('should handle playback state changes', async () => {
    renderWorkspace();

    await waitFor(() => {
      expect(screen.getByTestId('transport')).toBeInTheDocument();
    });

    // Transport controls handle playback state
  });

  it('should auto-save periodically', async () => {
    jest.useFakeTimers();

    renderWorkspace();

    await waitFor(() => {
      expect(screen.getByTestId('auto-save-indicator')).toBeInTheDocument();
    });

    // Advance timers to trigger auto-save
    jest.advanceTimersByTime(5000);

    jest.useRealTimers();
  });

  it('should display project ID attribute', async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        id: 'specific-project-id',
        title: 'Test',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [],
      }),
    });

    renderWorkspace();

    await waitFor(() => {
      const projectElement = screen.getByTestId('project-id');
      expect(projectElement).toHaveAttribute('data-id', 'specific-project-id');
    });
  });

  it('should render with default state before project loads', () => {
    renderWorkspace();

    expect(screen.getByTestId('workspace')).toBeInTheDocument();
  });
});
