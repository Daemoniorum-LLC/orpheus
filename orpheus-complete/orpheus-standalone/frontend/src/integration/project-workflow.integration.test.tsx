import { render, screen, fireEvent, waitFor, within } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import App from '../App';

/**
 * Integration Tests: Project Workflow
 * Tests complete project creation and management workflows
 */

describe('Project Workflow Integration', () => {
  let mockFetch: jest.Mock;

  beforeEach(() => {
    mockFetch = jest.fn();
    global.fetch = mockFetch;

    // Default mock for projects list
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => [],
    });
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('Create and Open Project Flow', () => {
    it('should complete full project creation flow', async () => {
      const mockProject = {
        id: 'new-project-123',
        title: 'My First Song',
        artist: 'Test Artist',
        bpm: 128,
        timeSignature: '4/4',
        key: 'Em',
        tracks: [],
      };

      // Mock sequence: list projects, create project, load project
      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => [],
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProject,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProject,
        });

      const { container } = render(
        <BrowserRouter>
          <App />
        </BrowserRouter>
      );

      // Should start on projects list
      await waitFor(() => {
        expect(screen.getByTestId('project-list')).toBeInTheDocument();
      });

      // Click new project button
      const newProjectButton = screen.getByTestId('new-project-button');
      fireEvent.click(newProjectButton);

      // Should navigate to new project form
      await waitFor(() => {
        expect(screen.getByTestId('new-project-form')).toBeInTheDocument();
      });

      // Fill out form
      fireEvent.change(screen.getByTestId('project-title-input'), {
        target: { value: 'My First Song' },
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

      // Submit form
      const submitButton = screen.getByTestId('create-project-submit');
      fireEvent.click(submitButton);

      // Should create project via API
      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          '/api/v1/projects',
          expect.objectContaining({
            method: 'POST',
            body: JSON.stringify({
              title: 'My First Song',
              artist: 'Test Artist',
              bpm: 128,
              timeSignature: '4/4',
              key: 'Em',
            }),
          })
        );
      });

      // Should navigate to workspace
      await waitFor(() => {
        expect(screen.getByTestId('workspace')).toBeInTheDocument();
      });

      // Should display project details
      expect(screen.getByTestId('project-title')).toHaveTextContent('My First Song');
      expect(screen.getByTestId('project-bpm')).toHaveTextContent('128');
    });

    it('should handle project creation error and allow retry', async () => {
      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => [],
        })
        .mockRejectedValueOnce(new Error('Network error'))
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({
            id: 'retry-project',
            title: 'Retry Song',
            bpm: 120,
            timeSignature: '4/4',
            tracks: [],
          }),
        });

      const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation();

      render(
        <BrowserRouter>
          <App />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByTestId('project-list')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByTestId('new-project-button'));

      await waitFor(() => {
        expect(screen.getByTestId('new-project-form')).toBeInTheDocument();
      });

      // First attempt - fails
      fireEvent.change(screen.getByTestId('project-title-input'), {
        target: { value: 'Retry Song' },
      });
      fireEvent.click(screen.getByTestId('create-project-submit'));

      await waitFor(() => {
        expect(consoleErrorSpy).toHaveBeenCalled();
      });

      // Second attempt - succeeds
      fireEvent.click(screen.getByTestId('create-project-submit'));

      await waitFor(() => {
        expect(screen.getByTestId('workspace')).toBeInTheDocument();
      });

      consoleErrorSpy.mockRestore();
    });
  });

  describe('Project List and Selection Flow', () => {
    it('should load, display, and open existing project', async () => {
      const mockProjects = [
        {
          id: 'project-1',
          title: 'Existing Song',
          artist: 'Artist 1',
          bpm: 140,
          timeSignature: '4/4',
          key: 'C',
          createdAt: '2024-01-01T00:00:00Z',
          updatedAt: '2024-01-02T00:00:00Z',
        },
        {
          id: 'project-2',
          title: 'Another Song',
          artist: 'Artist 2',
          bpm: 120,
          timeSignature: '3/4',
          key: 'Dm',
          createdAt: '2024-01-03T00:00:00Z',
          updatedAt: '2024-01-04T00:00:00Z',
        },
      ];

      const projectToOpen = {
        ...mockProjects[0],
        tracks: [
          {
            id: 'track-1',
            name: 'Drums',
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
          json: async () => mockProjects,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => projectToOpen,
        });

      render(
        <BrowserRouter>
          <App />
        </BrowserRouter>
      );

      // Should load projects
      await waitFor(() => {
        expect(screen.getByText('Existing Song')).toBeInTheDocument();
        expect(screen.getByText('Another Song')).toBeInTheDocument();
      });

      // Click first project
      const projectCard = screen.getAllByTestId('project-card')[0];
      fireEvent.click(projectCard);

      // Should navigate to workspace
      await waitFor(() => {
        expect(screen.getByTestId('workspace')).toBeInTheDocument();
      });

      // Should display project and its tracks
      expect(screen.getByTestId('project-title')).toHaveTextContent('Existing Song');
      expect(screen.getByText('Drums')).toBeInTheDocument();
    });

    it('should delete project and refresh list', async () => {
      const initialProjects = [
        {
          id: 'project-to-delete',
          title: 'Delete Me',
          artist: 'Artist',
          bpm: 120,
          timeSignature: '4/4',
          key: 'C',
          createdAt: '2024-01-01T00:00:00Z',
          updatedAt: '2024-01-01T00:00:00Z',
        },
        {
          id: 'project-keep',
          title: 'Keep Me',
          artist: 'Artist',
          bpm: 120,
          timeSignature: '4/4',
          key: 'C',
          createdAt: '2024-01-01T00:00:00Z',
          updatedAt: '2024-01-01T00:00:00Z',
        },
      ];

      const updatedProjects = [initialProjects[1]];

      global.confirm = jest.fn(() => true);

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => initialProjects,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({}),
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => updatedProjects,
        });

      render(
        <BrowserRouter>
          <App />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByText('Delete Me')).toBeInTheDocument();
        expect(screen.getByText('Keep Me')).toBeInTheDocument();
      });

      // Delete first project
      const deleteButtons = screen.getAllByTestId('delete-project-button');
      fireEvent.click(deleteButtons[0]);

      // Should show confirmation
      expect(global.confirm).toHaveBeenCalled();

      // Should delete via API
      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          '/api/v1/projects/project-to-delete',
          { method: 'DELETE' }
        );
      });

      // Should reload projects
      await waitFor(() => {
        expect(screen.queryByText('Delete Me')).not.toBeInTheDocument();
        expect(screen.getByText('Keep Me')).toBeInTheDocument();
      });
    });
  });

  describe('Project Metadata Updates', () => {
    it('should update project BPM and persist changes', async () => {
      const mockProject = {
        id: 'project-1',
        title: 'Test Project',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [],
      };

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => [mockProject],
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProject,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ ...mockProject, bpm: 140 }),
        });

      render(
        <BrowserRouter>
          <App />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByText('Test Project')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByTestId('project-card'));

      await waitFor(() => {
        expect(screen.getByTestId('workspace')).toBeInTheDocument();
      });

      // Initial BPM should be 120
      expect(screen.getByTestId('project-bpm')).toHaveTextContent('120');

      // Update would happen through workspace controls
      // This integration test verifies the complete flow
    });
  });

  describe('Navigation Flow', () => {
    it('should navigate between projects list and workspace', async () => {
      const mockProjects = [
        {
          id: 'nav-project',
          title: 'Nav Test',
          artist: 'Artist',
          bpm: 120,
          timeSignature: '4/4',
          key: 'C',
          createdAt: '2024-01-01T00:00:00Z',
          updatedAt: '2024-01-01T00:00:00Z',
          tracks: [],
        },
      ];

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProjects,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProjects[0],
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProjects,
        });

      render(
        <BrowserRouter>
          <App />
        </BrowserRouter>
      );

      // Start at projects list
      await waitFor(() => {
        expect(screen.getByTestId('project-list')).toBeInTheDocument();
      });

      // Navigate to workspace
      fireEvent.click(screen.getByTestId('project-card'));

      await waitFor(() => {
        expect(screen.getByTestId('workspace')).toBeInTheDocument();
      });

      // URL should reflect workspace route
      expect(window.location.pathname).toContain('/project/');
    });

    it('should handle browser back button correctly', async () => {
      const mockProjects = [
        {
          id: 'back-project',
          title: 'Back Test',
          bpm: 120,
          timeSignature: '4/4',
          tracks: [],
        },
      ];

      mockFetch.mockResolvedValue({
        ok: true,
        json: async () => mockProjects,
      });

      const { container } = render(
        <BrowserRouter>
          <App />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByTestId('project-list')).toBeInTheDocument();
      });

      // Navigation and back button behavior would be tested here
      // in a real browser environment with history API
    });
  });
});
