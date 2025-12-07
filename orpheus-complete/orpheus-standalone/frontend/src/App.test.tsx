import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import App from './App';

global.fetch = jest.fn();

describe('App', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    (global.fetch as jest.Mock).mockResolvedValue({
      ok: true,
      json: async () => ({ tracks: [] }),
    });
  });

  it('should render app container', () => {
    render(
      <MemoryRouter initialEntries={['/projects']}>
        <App />
      </MemoryRouter>
    );

    expect(screen.getByTestId('app')).toBeInTheDocument();
  });

  it('should redirect from root to /projects', async () => {
    render(
      <MemoryRouter initialEntries={['/']}>
        <App />
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(screen.getByTestId('project-list')).toBeInTheDocument();
    });
  });

  it('should render ProjectList on /projects route', () => {
    render(
      <MemoryRouter initialEntries={['/projects']}>
        <App />
      </MemoryRouter>
    );

    expect(screen.getByTestId('project-list')).toBeInTheDocument();
  });

  it('should render NewProject on /projects/new route', () => {
    render(
      <MemoryRouter initialEntries={['/projects/new']}>
        <App />
      </MemoryRouter>
    );

    expect(screen.getByTestId('new-project-form')).toBeInTheDocument();
  });

  it('should render Workspace on /project/:projectId route', async () => {
    const mockProject = {
      id: 'test-project-id',
      title: 'Test Project',
      bpm: 120,
      timeSignature: '4/4',
      tracks: [],
    };

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockProject,
    });

    render(
      <MemoryRouter initialEntries={['/project/test-project-id']}>
        <App />
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(screen.getByTestId('workspace')).toBeInTheDocument();
    });
  });

  it('should handle navigation between routes', async () => {
    const { rerender } = render(
      <MemoryRouter initialEntries={['/projects']}>
        <App />
      </MemoryRouter>
    );

    expect(screen.getByTestId('project-list')).toBeInTheDocument();

    rerender(
      <MemoryRouter initialEntries={['/projects/new']}>
        <App />
      </MemoryRouter>
    );

    expect(screen.getByTestId('new-project-form')).toBeInTheDocument();
  });

  it('should maintain router context', () => {
    render(
      <MemoryRouter initialEntries={['/projects']}>
        <App />
      </MemoryRouter>
    );

    // Router provides navigation context to child components
    expect(screen.getByTestId('app')).toBeInTheDocument();
  });
});
