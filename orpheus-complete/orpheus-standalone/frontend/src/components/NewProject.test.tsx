import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { NewProject } from './NewProject';

const mockNavigate = jest.fn();

jest.mock('react-router-dom', () => ({
  ...jest.requireActual('react-router-dom'),
  useNavigate: () => mockNavigate,
}));

global.fetch = jest.fn();

describe('NewProject', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  const renderNewProject = () => {
    return render(
      <BrowserRouter>
        <NewProject />
      </BrowserRouter>
    );
  };

  it('should render new project form', () => {
    renderNewProject();
    expect(screen.getByTestId('new-project-form')).toBeInTheDocument();
  });

  it('should render all form fields', () => {
    renderNewProject();

    expect(screen.getByTestId('project-title-input')).toBeInTheDocument();
    expect(screen.getByTestId('project-artist-input')).toBeInTheDocument();
    expect(screen.getByTestId('project-bpm-input')).toBeInTheDocument();
    expect(screen.getByTestId('project-time-signature-select')).toBeInTheDocument();
    expect(screen.getByTestId('project-key-input')).toBeInTheDocument();
  });

  it('should render submit and cancel buttons', () => {
    renderNewProject();

    expect(screen.getByTestId('create-project-submit')).toBeInTheDocument();
    expect(screen.getByTestId('cancel-button')).toBeInTheDocument();
  });

  it('should update title input', () => {
    renderNewProject();

    const titleInput = screen.getByTestId('project-title-input') as HTMLInputElement;
    fireEvent.change(titleInput, { target: { value: 'My New Song' } });

    expect(titleInput.value).toBe('My New Song');
  });

  it('should update artist input', () => {
    renderNewProject();

    const artistInput = screen.getByTestId('project-artist-input') as HTMLInputElement;
    fireEvent.change(artistInput, { target: { value: 'Test Artist' } });

    expect(artistInput.value).toBe('Test Artist');
  });

  it('should update BPM input', () => {
    renderNewProject();

    const bpmInput = screen.getByTestId('project-bpm-input') as HTMLInputElement;
    fireEvent.change(bpmInput, { target: { value: '140' } });

    expect(bpmInput.value).toBe('140');
  });

  it('should update time signature select', () => {
    renderNewProject();

    const timeSignatureSelect = screen.getByTestId('project-time-signature-select') as HTMLSelectElement;
    fireEvent.change(timeSignatureSelect, { target: { value: '3/4' } });

    expect(timeSignatureSelect.value).toBe('3/4');
  });

  it('should update key input', () => {
    renderNewProject();

    const keyInput = screen.getByTestId('project-key-input') as HTMLInputElement;
    fireEvent.change(keyInput, { target: { value: 'Em' } });

    expect(keyInput.value).toBe('Em');
  });

  it('should have default BPM of 120', () => {
    renderNewProject();

    const bpmInput = screen.getByTestId('project-bpm-input') as HTMLInputElement;
    expect(bpmInput.value).toBe('120');
  });

  it('should have default time signature of 4/4', () => {
    renderNewProject();

    const timeSignatureSelect = screen.getByTestId('project-time-signature-select') as HTMLSelectElement;
    expect(timeSignatureSelect.value).toBe('4/4');
  });

  it('should create project on form submit', async () => {
    const mockProject = {
      id: 'new-project-id',
      title: 'Test Song',
      artist: 'Test Artist',
      bpm: 128,
      timeSignature: '4/4',
      key: 'C',
    };

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockProject,
    });

    renderNewProject();

    fireEvent.change(screen.getByTestId('project-title-input'), {
      target: { value: 'Test Song' },
    });
    fireEvent.change(screen.getByTestId('project-artist-input'), {
      target: { value: 'Test Artist' },
    });
    fireEvent.change(screen.getByTestId('project-bpm-input'), {
      target: { value: '128' },
    });
    fireEvent.change(screen.getByTestId('project-key-input'), {
      target: { value: 'C' },
    });

    const submitButton = screen.getByTestId('create-project-submit');
    fireEvent.click(submitButton);

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith('/api/v1/projects', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          title: 'Test Song',
          artist: 'Test Artist',
          bpm: 128,
          timeSignature: '4/4',
          key: 'C',
        }),
      });
    });

    expect(mockNavigate).toHaveBeenCalledWith('/project/new-project-id');
  });

  it('should navigate to projects list on cancel', () => {
    renderNewProject();

    const cancelButton = screen.getByTestId('cancel-button');
    fireEvent.click(cancelButton);

    expect(mockNavigate).toHaveBeenCalledWith('/projects');
  });

  it('should validate required title field', async () => {
    renderNewProject();

    const submitButton = screen.getByTestId('create-project-submit');
    fireEvent.click(submitButton);

    // Should not make API call without title
    expect(global.fetch).not.toHaveBeenCalled();
  });

  it('should validate BPM range (min)', () => {
    renderNewProject();

    const bpmInput = screen.getByTestId('project-bpm-input') as HTMLInputElement;
    expect(bpmInput.min).toBe('20');
  });

  it('should validate BPM range (max)', () => {
    renderNewProject();

    const bpmInput = screen.getByTestId('project-bpm-input') as HTMLInputElement;
    expect(bpmInput.max).toBe('300');
  });

  it('should include all time signature options', () => {
    renderNewProject();

    const timeSignatureSelect = screen.getByTestId('project-time-signature-select');
    const options = timeSignatureSelect.querySelectorAll('option');

    const values = Array.from(options).map(opt => opt.getAttribute('value'));

    expect(values).toContain('4/4');
    expect(values).toContain('3/4');
    expect(values).toContain('6/8');
    expect(values).toContain('7/8');
    expect(values).toContain('5/4');
  });

  it('should handle API errors gracefully', async () => {
    const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation();

    (global.fetch as jest.Mock).mockRejectedValueOnce(new Error('API Error'));

    renderNewProject();

    fireEvent.change(screen.getByTestId('project-title-input'), {
      target: { value: 'Test Song' },
    });

    const submitButton = screen.getByTestId('create-project-submit');
    fireEvent.click(submitButton);

    await waitFor(() => {
      expect(consoleErrorSpy).toHaveBeenCalled();
    });

    consoleErrorSpy.mockRestore();
  });

  it('should submit form on Enter key in title input', async () => {
    const mockProject = {
      id: 'new-project-id',
      title: 'Test Song',
      artist: '',
      bpm: 120,
      timeSignature: '4/4',
      key: '',
    };

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockProject,
    });

    renderNewProject();

    const titleInput = screen.getByTestId('project-title-input');
    fireEvent.change(titleInput, { target: { value: 'Test Song' } });

    const form = screen.getByTestId('new-project-form');
    fireEvent.submit(form);

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalled();
    });
  });

  it('should handle non-numeric BPM input', () => {
    renderNewProject();

    const bpmInput = screen.getByTestId('project-bpm-input') as HTMLInputElement;
    expect(bpmInput.type).toBe('number');
  });

  it('should allow empty artist field', async () => {
    const mockProject = {
      id: 'new-project-id',
      title: 'Test Song',
      artist: '',
      bpm: 120,
      timeSignature: '4/4',
      key: '',
    };

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockProject,
    });

    renderNewProject();

    fireEvent.change(screen.getByTestId('project-title-input'), {
      target: { value: 'Test Song' },
    });

    const submitButton = screen.getByTestId('create-project-submit');
    fireEvent.click(submitButton);

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalled();
    });
  });

  it('should allow empty key field', async () => {
    const mockProject = {
      id: 'new-project-id',
      title: 'Test Song',
      artist: 'Artist',
      bpm: 120,
      timeSignature: '4/4',
      key: '',
    };

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockProject,
    });

    renderNewProject();

    fireEvent.change(screen.getByTestId('project-title-input'), {
      target: { value: 'Test Song' },
    });
    fireEvent.change(screen.getByTestId('project-artist-input'), {
      target: { value: 'Artist' },
    });

    const submitButton = screen.getByTestId('create-project-submit');
    fireEvent.click(submitButton);

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalled();
    });
  });
});
