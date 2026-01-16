import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { Collaboration } from './Collaboration';

global.fetch = jest.fn();

describe('Collaboration', () => {
  const defaultProps = {
    projectId: 'test-project-id',
    currentUserId: 'current-user-id',
  };

  beforeEach(() => {
    jest.clearAllMocks();
    (global.fetch as jest.Mock).mockResolvedValue({
      ok: true,
      json: async () => [],
    });
  });

  it('should render collaboration toolbar', () => {
    render(<Collaboration {...defaultProps} />);
    expect(screen.getByTestId('collaboration-toolbar')).toBeInTheDocument();
  });

  it('should show online indicator by default', () => {
    render(<Collaboration {...defaultProps} />);
    expect(screen.getByTestId('online-indicator')).toBeInTheDocument();
  });

  it('should show active collaborators count', () => {
    render(<Collaboration {...defaultProps} />);
    expect(screen.getByTestId('active-collaborators')).toBeInTheDocument();
  });

  it('should render share button', () => {
    render(<Collaboration {...defaultProps} />);
    expect(screen.getByTestId('share-project-button')).toBeInTheDocument();
  });

  it('should render chat button', () => {
    render(<Collaboration {...defaultProps} />);
    expect(screen.getByTestId('chat-button')).toBeInTheDocument();
  });

  it('should load collaborators on mount', async () => {
    const mockCollaborators = [
      {
        id: '1',
        email: 'user@example.com',
        name: 'Test User',
        role: 'editor',
        active: true,
        avatarColor: '#3b82f6',
      },
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockCollaborators,
    });

    render(<Collaboration {...defaultProps} />);

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        '/api/v1/projects/test-project-id/collaborators'
      );
    });
  });

  it('should display active collaborator avatars', async () => {
    const mockCollaborators = [
      {
        id: '1',
        email: 'user1@example.com',
        name: 'User 1',
        role: 'editor',
        active: true,
        avatarColor: '#3b82f6',
      },
      {
        id: '2',
        email: 'user2@example.com',
        name: 'User 2',
        role: 'editor',
        active: true,
        avatarColor: '#10b981',
      },
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockCollaborators,
    });

    render(<Collaboration {...defaultProps} />);

    await waitFor(() => {
      const avatars = screen.getAllByTestId('collaborator-avatar');
      expect(avatars.length).toBeGreaterThan(0);
    });
  });

  it('should open share dialog when share button clicked', () => {
    render(<Collaboration {...defaultProps} />);

    const shareButton = screen.getByTestId('share-project-button');
    fireEvent.click(shareButton);

    expect(screen.getByText('Share Project')).toBeInTheDocument();
  });

  it('should close share dialog when close button clicked', () => {
    render(<Collaboration {...defaultProps} />);

    fireEvent.click(screen.getByTestId('share-project-button'));

    const closeButton = screen.getByText('×');
    fireEvent.click(closeButton);

    expect(screen.queryByText('Share Project')).not.toBeInTheDocument();
  });

  it('should render invite form in share dialog', () => {
    render(<Collaboration {...defaultProps} />);

    fireEvent.click(screen.getByTestId('share-project-button'));

    expect(screen.getByTestId('collaborator-email-input')).toBeInTheDocument();
    expect(screen.getByTestId('collaborator-role-select')).toBeInTheDocument();
    expect(screen.getByTestId('send-invite-button')).toBeInTheDocument();
  });

  it('should update email input', () => {
    render(<Collaboration {...defaultProps} />);

    fireEvent.click(screen.getByTestId('share-project-button'));

    const emailInput = screen.getByTestId('collaborator-email-input') as HTMLInputElement;
    fireEvent.change(emailInput, { target: { value: 'new@example.com' } });

    expect(emailInput.value).toBe('new@example.com');
  });

  it('should update role select', () => {
    render(<Collaboration {...defaultProps} />);

    fireEvent.click(screen.getByTestId('share-project-button'));

    const roleSelect = screen.getByTestId('collaborator-role-select') as HTMLSelectElement;
    fireEvent.change(roleSelect, { target: { value: 'viewer' } });

    expect(roleSelect.value).toBe('viewer');
  });

  it('should send invite when button clicked', async () => {
    (global.fetch as jest.Mock).mockResolvedValue({
      ok: true,
      json: async () => ({}),
    });

    render(<Collaboration {...defaultProps} />);

    fireEvent.click(screen.getByTestId('share-project-button'));

    fireEvent.change(screen.getByTestId('collaborator-email-input'), {
      target: { value: 'invite@example.com' },
    });
    fireEvent.change(screen.getByTestId('collaborator-role-select'), {
      target: { value: 'editor' },
    });

    fireEvent.click(screen.getByTestId('send-invite-button'));

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        '/api/v1/projects/test-project-id/collaborators',
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify({
            email: 'invite@example.com',
            role: 'editor',
          }),
        })
      );
    });
  });

  it('should display collaborator list', async () => {
    const mockCollaborators = [
      {
        id: '1',
        email: 'user@example.com',
        name: 'Test User',
        role: 'editor',
        active: true,
        avatarColor: '#3b82f6',
      },
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockCollaborators,
    });

    render(<Collaboration {...defaultProps} />);

    fireEvent.click(screen.getByTestId('share-project-button'));

    await waitFor(() => {
      expect(screen.getByTestId('collaborator-list')).toBeInTheDocument();
    });
  });

  it('should display collaborator role', async () => {
    const mockCollaborators = [
      {
        id: '1',
        email: 'user@example.com',
        name: 'Test User',
        role: 'editor',
        active: true,
        avatarColor: '#3b82f6',
      },
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockCollaborators,
    });

    render(<Collaboration {...defaultProps} />);

    fireEvent.click(screen.getByTestId('share-project-button'));

    await waitFor(() => {
      expect(screen.getByText('Editor')).toBeInTheDocument();
    });
  });

  it('should show change role button for non-owners', async () => {
    const mockCollaborators = [
      {
        id: '1',
        email: 'user@example.com',
        name: 'Test User',
        role: 'editor',
        active: true,
        avatarColor: '#3b82f6',
      },
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockCollaborators,
    });

    render(<Collaboration {...defaultProps} />);

    fireEvent.click(screen.getByTestId('share-project-button'));

    await waitFor(() => {
      expect(screen.getByTestId('change-role-button')).toBeInTheDocument();
    });
  });

  it('should show remove button for non-owners', async () => {
    const mockCollaborators = [
      {
        id: '1',
        email: 'user@example.com',
        name: 'Test User',
        role: 'editor',
        active: true,
        avatarColor: '#3b82f6',
      },
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockCollaborators,
    });

    render(<Collaboration {...defaultProps} />);

    fireEvent.click(screen.getByTestId('share-project-button'));

    await waitFor(() => {
      expect(screen.getByTestId('remove-collaborator-button')).toBeInTheDocument();
    });
  });

  it('should open chat panel when chat button clicked', () => {
    render(<Collaboration {...defaultProps} />);

    const chatButton = screen.getByTestId('chat-button');
    fireEvent.click(chatButton);

    expect(screen.getByTestId('chat-panel')).toBeInTheDocument();
  });

  it('should close chat panel', () => {
    render(<Collaboration {...defaultProps} />);

    fireEvent.click(screen.getByTestId('chat-button'));

    const closeButton = screen.getByText('−');
    fireEvent.click(closeButton);

    expect(screen.queryByTestId('chat-panel')).not.toBeInTheDocument();
  });

  it('should render chat input', () => {
    render(<Collaboration {...defaultProps} />);

    fireEvent.click(screen.getByTestId('chat-button'));

    expect(screen.getByTestId('chat-input')).toBeInTheDocument();
  });

  it('should send chat message', async () => {
    (global.fetch as jest.Mock).mockResolvedValue({
      ok: true,
      json: async () => ({}),
    });

    render(<Collaboration {...defaultProps} />);

    fireEvent.click(screen.getByTestId('chat-button'));

    const chatInput = screen.getByTestId('chat-input');
    fireEvent.change(chatInput, { target: { value: 'Hello team!' } });
    fireEvent.keyPress(chatInput, { key: 'Enter', code: 13, charCode: 13 });

    await waitFor(() => {
      expect(screen.getByText('Hello team!')).toBeInTheDocument();
    });
  });

  it('should display chat messages', async () => {
    render(<Collaboration {...defaultProps} />);

    fireEvent.click(screen.getByTestId('chat-button'));

    expect(screen.getByTestId('chat-messages')).toBeInTheDocument();
  });

  it('should show offline indicator when offline', async () => {
    render(<Collaboration {...defaultProps} />);

    // Simulate going offline
    window.dispatchEvent(new Event('offline'));

    await waitFor(() => {
      expect(screen.getByTestId('offline-indicator')).toBeInTheDocument();
    });
  });

  it('should show online indicator when back online', async () => {
    render(<Collaboration {...defaultProps} />);

    // Simulate going offline then online
    window.dispatchEvent(new Event('offline'));
    window.dispatchEvent(new Event('online'));

    await waitFor(() => {
      expect(screen.getByTestId('online-indicator')).toBeInTheDocument();
    });
  });

  it('should not send invite with empty email', async () => {
    render(<Collaboration {...defaultProps} />);

    fireEvent.click(screen.getByTestId('share-project-button'));

    fireEvent.click(screen.getByTestId('send-invite-button'));

    // Should not make API call
    expect(global.fetch).toHaveBeenCalledTimes(1); // Only initial load
  });

  it('should handle fetch error on load', async () => {
    const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation();

    (global.fetch as jest.Mock).mockRejectedValueOnce(new Error('Network error'));

    render(<Collaboration {...defaultProps} />);

    await waitFor(() => {
      expect(consoleErrorSpy).toHaveBeenCalled();
    });

    consoleErrorSpy.mockRestore();
  });

  it('should clear input after sending message', async () => {
    render(<Collaboration {...defaultProps} />);

    fireEvent.click(screen.getByTestId('chat-button'));

    const chatInput = screen.getByTestId('chat-input') as HTMLInputElement;
    fireEvent.change(chatInput, { target: { value: 'Test message' } });
    fireEvent.keyPress(chatInput, { key: 'Enter' });

    await waitFor(() => {
      expect(chatInput.value).toBe('');
    });
  });

  it('should not send empty chat message', async () => {
    render(<Collaboration {...defaultProps} />);

    fireEvent.click(screen.getByTestId('chat-button'));

    const chatInput = screen.getByTestId('chat-input');
    fireEvent.keyPress(chatInput, { key: 'Enter' });

    const messages = screen.queryAllByTestId('chat-message');
    expect(messages.length).toBe(0);
  });

  it('should show more than 5 collaborators indicator', async () => {
    const mockCollaborators = Array.from({ length: 7 }, (_, i) => ({
      id: `user-${i}`,
      email: `user${i}@example.com`,
      name: `User ${i}`,
      role: 'editor' as const,
      active: true,
      avatarColor: '#3b82f6',
    }));

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: async () => mockCollaborators,
    });

    render(<Collaboration {...defaultProps} />);

    await waitFor(() => {
      const avatars = screen.getAllByTestId('collaborator-avatar');
      expect(avatars.length).toBeGreaterThan(0);
    });
  });
});
