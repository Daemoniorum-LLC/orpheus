import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { ProjectList } from './ProjectList';

const mockNavigate = jest.fn();

jest.mock('react-router-dom', () => ({
  ...jest.requireActual('react-router-dom'),
  useNavigate: () => mockNavigate,
}));

global.fetch = jest.fn();

describe('ProjectList', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    (global.fetch as jest.Mock).mockResolvedValue({
      ok: true,
      json: async () => [],
    });
  });

  const renderProjectList = () => {
    return render(
      <BrowserRouter>
        <ProjectList />
      </BrowserRouter>
    );
  };

  it('should render project list container', () => {
    renderProjectList();
    expect(screen.getByTestId('project-list')).toBeInTheDocument();
  });

  it('should render new project button', () => {
    renderProjectList();
    expect(screen.getByTestId('new-project-button')).toBeInTheDocument();
  });

  it('should load projects on mount', async () => {
    const mockProjects = [
      {
        id: '1',
        title: 'Project 1',
        artist: 'Artist 1',
        bpm: 120,
        timeSignature: '4/4',
        key: 'C',
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      },
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockProjects,
    });

    renderProjectList();

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith('/api/v1/projects');
    });

    await waitFor(() => {
      expect(screen.getByText('Project 1')).toBeInTheDocument();
    });
  });

  it('should display multiple projects', async () => {
    const mockProjects = [
      {
        id: '1',
        title: 'Project 1',
        artist: 'Artist 1',
        bpm: 120,
        timeSignature: '4/4',
        key: 'C',
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      },
      {
        id: '2',
        title: 'Project 2',
        artist: 'Artist 2',
        bpm: 140,
        timeSignature: '3/4',
        key: 'Em',
        createdAt: '2024-01-02T00:00:00Z',
        updatedAt: '2024-01-02T00:00:00Z',
      },
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockProjects,
    });

    renderProjectList();

    await waitFor(() => {
      expect(screen.getByText('Project 1')).toBeInTheDocument();
      expect(screen.getByText('Project 2')).toBeInTheDocument();
    });
  });

  it('should navigate to new project page when new project button clicked', () => {
    renderProjectList();

    const newProjectButton = screen.getByTestId('new-project-button');
    fireEvent.click(newProjectButton);

    expect(mockNavigate).toHaveBeenCalledWith('/projects/new');
  });

  it('should navigate to project workspace when project card clicked', async () => {
    const mockProjects = [
      {
        id: 'test-project-id',
        title: 'Test Project',
        artist: 'Test Artist',
        bpm: 120,
        timeSignature: '4/4',
        key: 'C',
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      },
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockProjects,
    });

    renderProjectList();

    await waitFor(() => {
      expect(screen.getByText('Test Project')).toBeInTheDocument();
    });

    const projectCard = screen.getByTestId('project-card');
    fireEvent.click(projectCard);

    expect(mockNavigate).toHaveBeenCalledWith('/project/test-project-id');
  });

  it('should show delete confirmation when delete button clicked', async () => {
    const mockProjects = [
      {
        id: '1',
        title: 'Project 1',
        artist: 'Artist 1',
        bpm: 120,
        timeSignature: '4/4',
        key: 'C',
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      },
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockProjects,
    });

    global.confirm = jest.fn(() => true);

    renderProjectList();

    await waitFor(() => {
      expect(screen.getByText('Project 1')).toBeInTheDocument();
    });

    const deleteButton = screen.getByTestId('delete-project-button');
    fireEvent.click(deleteButton);

    expect(global.confirm).toHaveBeenCalled();
  });

  it('should delete project when confirmed', async () => {
    const mockProjects = [
      {
        id: 'project-to-delete',
        title: 'Project 1',
        artist: 'Artist 1',
        bpm: 120,
        timeSignature: '4/4',
        key: 'C',
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      },
    ];

    (global.fetch as jest.Mock)
      .mockResolvedValueOnce({
        ok: true,
        json: async () => mockProjects,
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({}),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => [],
      });

    global.confirm = jest.fn(() => true);

    renderProjectList();

    await waitFor(() => {
      expect(screen.getByText('Project 1')).toBeInTheDocument();
    });

    const deleteButton = screen.getByTestId('delete-project-button');
    fireEvent.click(deleteButton);

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        '/api/v1/projects/project-to-delete',
        { method: 'DELETE' }
      );
    });
  });

  it('should not delete project when cancelled', async () => {
    const mockProjects = [
      {
        id: '1',
        title: 'Project 1',
        artist: 'Artist 1',
        bpm: 120,
        timeSignature: '4/4',
        key: 'C',
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      },
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockProjects,
    });

    global.confirm = jest.fn(() => false);

    renderProjectList();

    await waitFor(() => {
      expect(screen.getByText('Project 1')).toBeInTheDocument();
    });

    const deleteButton = screen.getByTestId('delete-project-button');
    const fetchCallCount = (global.fetch as jest.Mock).mock.calls.length;

    fireEvent.click(deleteButton);

    // Should not make additional fetch calls
    expect((global.fetch as jest.Mock).mock.calls.length).toBe(fetchCallCount);
  });

  it('should handle fetch error gracefully', async () => {
    const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation();

    (global.fetch as jest.Mock).mockRejectedValueOnce(new Error('Network error'));

    renderProjectList();

    await waitFor(() => {
      expect(consoleErrorSpy).toHaveBeenCalled();
    });

    consoleErrorSpy.mockRestore();
  });

  it('should display project metadata correctly', async () => {
    const mockProjects = [
      {
        id: '1',
        title: 'My Song',
        artist: 'Test Artist',
        bpm: 128,
        timeSignature: '7/8',
        key: 'Dm',
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-02T00:00:00Z',
      },
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockProjects,
    });

    renderProjectList();

    await waitFor(() => {
      expect(screen.getByText('My Song')).toBeInTheDocument();
      expect(screen.getByText('Test Artist')).toBeInTheDocument();
      expect(screen.getByText(/128 BPM/)).toBeInTheDocument();
      expect(screen.getByText(/7\/8/)).toBeInTheDocument();
      expect(screen.getByText(/Dm/)).toBeInTheDocument();
    });
  });

  it('should format dates correctly', async () => {
    const mockProjects = [
      {
        id: '1',
        title: 'Project 1',
        artist: 'Artist 1',
        bpm: 120,
        timeSignature: '4/4',
        key: 'C',
        createdAt: '2024-01-15T10:30:00Z',
        updatedAt: '2024-01-16T14:45:00Z',
      },
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockProjects,
    });

    renderProjectList();

    await waitFor(() => {
      expect(screen.getByText('Project 1')).toBeInTheDocument();
    });

    const dateElements = screen.getAllByText(/1\/1/);
    expect(dateElements.length).toBeGreaterThan(0);
  });

  it('should show empty state when no projects exist', async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => [],
    });

    renderProjectList();

    await waitFor(() => {
      expect(screen.getByTestId('project-list')).toBeInTheDocument();
    });

    const projectCards = screen.queryAllByTestId('project-card');
    expect(projectCards.length).toBe(0);
  });
});
