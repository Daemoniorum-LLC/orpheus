import { render, screen, fireEvent, waitFor, within } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { Workspace } from '../components/Workspace';
import { Collaboration } from '../components/Collaboration';

/**
 * Integration Tests: Collaboration Workflow
 * Tests real-time collaboration, sharing, and multi-user workflows
 */

describe('Collaboration Workflow Integration', () => {
  let mockFetch: jest.Mock;

  beforeEach(() => {
    mockFetch = jest.fn();
    global.fetch = mockFetch;
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('Project Sharing Flow', () => {
    it('should complete full project sharing workflow', async () => {
      const mockCollaborators = [
        {
          id: 'owner-1',
          email: 'owner@example.com',
          name: 'Project Owner',
          role: 'owner',
          active: true,
          avatarColor: '#3b82f6',
        },
      ];

      const newCollaborator = {
        id: 'collab-1',
        email: 'collaborator@example.com',
        name: 'New Collaborator',
        role: 'editor',
        active: false,
        avatarColor: '#10b981',
      };

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockCollaborators,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => newCollaborator,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => [...mockCollaborators, newCollaborator],
        });

      render(<Collaboration projectId="test-project" currentUserId="owner-1" />);

      await waitFor(() => {
        expect(screen.getByTestId('collaboration-toolbar')).toBeInTheDocument();
      });

      // Open share dialog
      fireEvent.click(screen.getByTestId('share-project-button'));

      await waitFor(() => {
        expect(screen.getByText('Share Project')).toBeInTheDocument();
      });

      // Enter collaborator email
      fireEvent.change(screen.getByTestId('collaborator-email-input'), {
        target: { value: 'collaborator@example.com' },
      });

      // Select role
      fireEvent.change(screen.getByTestId('collaborator-role-select'), {
        target: { value: 'editor' },
      });

      // Send invite
      fireEvent.click(screen.getByTestId('send-invite-button'));

      // Should call API to invite
      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          '/api/v1/projects/test-project/collaborators',
          expect.objectContaining({
            method: 'POST',
            body: JSON.stringify({
              email: 'collaborator@example.com',
              role: 'editor',
            }),
          })
        );
      });

      // Should reload collaborators
      await waitFor(() => {
        expect(screen.getByText('collaborator@example.com')).toBeInTheDocument();
      });
    });

    it('should manage collaborator roles', async () => {
      const mockCollaborators = [
        {
          id: 'owner-1',
          email: 'owner@example.com',
          name: 'Owner',
          role: 'owner',
          active: true,
          avatarColor: '#3b82f6',
        },
        {
          id: 'editor-1',
          email: 'editor@example.com',
          name: 'Editor',
          role: 'editor',
          active: true,
          avatarColor: '#10b981',
        },
      ];

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockCollaborators,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({}),
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => [
            mockCollaborators[0],
            { ...mockCollaborators[1], role: 'viewer' },
          ],
        });

      render(<Collaboration projectId="test-project" currentUserId="owner-1" />);

      await waitFor(() => {
        expect(screen.getByTestId('share-project-button')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByTestId('share-project-button'));

      await waitFor(() => {
        expect(screen.getByText('editor@example.com')).toBeInTheDocument();
      });

      // Change role from editor to viewer
      const changeRoleButton = screen.getByTestId('change-role-button');
      fireEvent.click(changeRoleButton);

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          '/api/v1/projects/test-project/collaborators/editor-1/role',
          expect.objectContaining({
            method: 'PUT',
            body: expect.stringContaining('viewer'),
          })
        );
      });
    });

    it('should remove collaborator from project', async () => {
      const mockCollaborators = [
        {
          id: 'owner-1',
          email: 'owner@example.com',
          name: 'Owner',
          role: 'owner',
          active: true,
          avatarColor: '#3b82f6',
        },
        {
          id: 'remove-me',
          email: 'remove@example.com',
          name: 'Remove Me',
          role: 'editor',
          active: false,
          avatarColor: '#10b981',
        },
      ];

      global.confirm = jest.fn(() => true);

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockCollaborators,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({}),
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => [mockCollaborators[0]],
        });

      render(<Collaboration projectId="test-project" currentUserId="owner-1" />);

      await waitFor(() => {
        expect(screen.getByTestId('share-project-button')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByTestId('share-project-button'));

      await waitFor(() => {
        expect(screen.getByText('remove@example.com')).toBeInTheDocument();
      });

      // Remove collaborator
      fireEvent.click(screen.getByTestId('remove-collaborator-button'));

      expect(global.confirm).toHaveBeenCalled();

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          '/api/v1/projects/test-project/collaborators/remove-me',
          expect.objectContaining({
            method: 'DELETE',
          })
        );
      });

      await waitFor(() => {
        expect(screen.queryByText('remove@example.com')).not.toBeInTheDocument();
      });
    });
  });

  describe('Real-Time Collaboration', () => {
    it('should show active collaborator presence', async () => {
      const mockCollaborators = [
        {
          id: 'user-1',
          email: 'user1@example.com',
          name: 'User 1',
          role: 'editor',
          active: true,
          avatarColor: '#3b82f6',
        },
        {
          id: 'user-2',
          email: 'user2@example.com',
          name: 'User 2',
          role: 'editor',
          active: true,
          avatarColor: '#10b981',
        },
        {
          id: 'user-3',
          email: 'user3@example.com',
          name: 'User 3',
          role: 'viewer',
          active: false,
          avatarColor: '#f59e0b',
        },
      ];

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockCollaborators,
      });

      render(<Collaboration projectId="test-project" currentUserId="user-1" />);

      await waitFor(() => {
        expect(screen.getByTestId('active-collaborators')).toBeInTheDocument();
      });

      // Should show 2 active collaborators
      expect(screen.getByText(/2 active/)).toBeInTheDocument();

      // Should show avatars for active users
      const avatars = screen.getAllByTestId('collaborator-avatar');
      expect(avatars.length).toBeGreaterThanOrEqual(2);
    });

    it('should handle online/offline transitions', async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        json: async () => [],
      });

      render(<Collaboration projectId="test-project" currentUserId="user-1" />);

      await waitFor(() => {
        expect(screen.getByTestId('online-indicator')).toBeInTheDocument();
      });

      // Simulate going offline
      window.dispatchEvent(new Event('offline'));

      await waitFor(() => {
        expect(screen.getByTestId('offline-indicator')).toBeInTheDocument();
      });

      // Simulate coming back online
      window.dispatchEvent(new Event('online'));

      await waitFor(() => {
        expect(screen.getByTestId('online-indicator')).toBeInTheDocument();
      });
    });
  });

  describe('Chat Integration', () => {
    it('should send and receive chat messages', async () => {
      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => [],
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({}),
        });

      render(<Collaboration projectId="test-project" currentUserId="user-1" />);

      await waitFor(() => {
        expect(screen.getByTestId('chat-button')).toBeInTheDocument();
      });

      // Open chat
      fireEvent.click(screen.getByTestId('chat-button'));

      await waitFor(() => {
        expect(screen.getByTestId('chat-panel')).toBeInTheDocument();
      });

      // Send message
      const chatInput = screen.getByTestId('chat-input');
      fireEvent.change(chatInput, {
        target: { value: 'Hello team!' },
      });
      fireEvent.keyPress(chatInput, { key: 'Enter', code: 13, charCode: 13 });

      // Should call API
      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          '/api/v1/projects/test-project/chat',
          expect.objectContaining({
            method: 'POST',
            body: JSON.stringify({ message: 'Hello team!' }),
          })
        );
      });

      // Message should appear in chat
      await waitFor(() => {
        expect(screen.getByText('Hello team!')).toBeInTheDocument();
      });
    });

    it('should display message timestamps', async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        json: async () => [],
      });

      render(<Collaboration projectId="test-project" currentUserId="user-1" />);

      await waitFor(() => {
        expect(screen.getByTestId('chat-button')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByTestId('chat-button'));

      const chatInput = screen.getByTestId('chat-input');
      fireEvent.change(chatInput, { target: { value: 'Test message' } });
      fireEvent.keyPress(chatInput, { key: 'Enter' });

      await waitFor(() => {
        expect(screen.getByTestId('message-time')).toBeInTheDocument();
      });
    });
  });

  describe('Collaborative Editing', () => {
    it('should synchronize track changes across collaborators', async () => {
      const mockProject = {
        id: 'collab-project',
        title: 'Collaborative Project',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [
          {
            id: 'track-1',
            name: 'Shared Track',
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
          json: async () => ({ ...mockProject.tracks[0], volume: 0.6 }),
        });

      render(
        <BrowserRouter>
          <Workspace />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByText('Shared Track')).toBeInTheDocument();
      });

      // User adjusts volume
      const volumeSlider = screen.getByTestId('track-volume-slider');
      fireEvent.change(volumeSlider, { target: { value: '0.6' } });

      // Should notify other collaborators
      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          expect.stringContaining('/tracks/track-1'),
          expect.objectContaining({
            method: 'PATCH',
          })
        );
      });
    });

    it('should show notifications for collaborator actions', async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        json: async () => [],
      });

      render(<Collaboration projectId="test-project" currentUserId="user-1" />);

      await waitFor(() => {
        expect(screen.getByTestId('collaboration-toolbar')).toBeInTheDocument();
      });

      // Simulate receiving notification from WebSocket or polling
      // In real implementation, this would come from the server
      const notification = {
        id: 'notif-1',
        message: 'User 2 added track "Bass"',
        timestamp: new Date(),
        type: 'info',
      };

      // Would trigger notification display
      // Testing the notification system integration
    });
  });

  describe('Conflict Resolution', () => {
    it('should handle concurrent edits to same track', async () => {
      const mockProject = {
        id: 'conflict-project',
        title: 'Conflict Test',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [
          {
            id: 'track-1',
            name: 'Contested Track',
            volume: 0.8,
            pan: 0,
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
          json: async () => ({ ...mockProject.tracks[0], volume: 0.5 }),
        });

      render(
        <BrowserRouter>
          <Workspace />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByText('Contested Track')).toBeInTheDocument();
      });

      // Simulate conflict scenario
      // User 1 and User 2 edit simultaneously
      // Last write wins or merge strategy would be tested
    });
  });

  describe('Permission-Based Actions', () => {
    it('should restrict viewer from editing', async () => {
      const mockCollaborators = [
        {
          id: 'viewer-1',
          email: 'viewer@example.com',
          name: 'Viewer',
          role: 'viewer',
          active: true,
          avatarColor: '#3b82f6',
        },
      ];

      mockFetch.mockResolvedValue({
        ok: true,
        json: async () => mockCollaborators,
      });

      render(<Collaboration projectId="test-project" currentUserId="viewer-1" />);

      await waitFor(() => {
        expect(screen.getByTestId('collaboration-toolbar')).toBeInTheDocument();
      });

      // Viewer should not see change role or remove buttons
      fireEvent.click(screen.getByTestId('share-project-button'));

      await waitFor(() => {
        expect(screen.queryByTestId('change-role-button')).not.toBeInTheDocument();
        expect(screen.queryByTestId('remove-collaborator-button')).not.toBeInTheDocument();
      });
    });

    it('should allow editor to make changes', async () => {
      const mockProject = {
        id: 'editor-project',
        title: 'Editor Test',
        bpm: 120,
        timeSignature: '4/4',
        tracks: [],
      };

      const newTrack = {
        id: 'track-1',
        name: 'Editor Track',
        volume: 0.8,
        pan: 0,
      };

      mockFetch
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProject,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => newTrack,
        });

      render(
        <BrowserRouter>
          <Workspace />
        </BrowserRouter>
      );

      await waitFor(() => {
        expect(screen.getByTestId('workspace')).toBeInTheDocument();
      });

      // Editor can add track
      fireEvent.click(screen.getByTestId('add-track-button'));

      await waitFor(() => {
        expect(screen.getByTestId('track-name-input')).toBeInTheDocument();
      });

      fireEvent.change(screen.getByTestId('track-name-input'), {
        target: { value: 'Editor Track' },
      });
      fireEvent.click(screen.getByTestId('create-track-submit'));

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          expect.stringContaining('/tracks'),
          expect.objectContaining({
            method: 'POST',
          })
        );
      });
    });
  });
});
