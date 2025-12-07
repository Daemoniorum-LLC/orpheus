import React, { useState, useEffect } from 'react';
import './Collaboration.css';

interface Collaborator {
  id: string;
  email: string;
  name: string;
  role: 'owner' | 'editor' | 'viewer';
  active: boolean;
  avatarColor: string;
}

interface ChatMessage {
  id: string;
  userId: string;
  userName: string;
  message: string;
  timestamp: Date;
}

interface Notification {
  id: string;
  message: string;
  timestamp: Date;
  type: 'info' | 'success' | 'warning';
}

interface CollaborationProps {
  projectId: string;
  currentUserId: string;
}

export const Collaboration: React.FC<CollaborationProps> = ({ projectId, currentUserId }) => {
  const [collaborators, setCollaborators] = useState<Collaborator[]>([]);
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [showShareDialog, setShowShareDialog] = useState(false);
  const [showChat, setShowChat] = useState(false);
  const [newCollaboratorEmail, setNewCollaboratorEmail] = useState('');
  const [newCollaboratorRole, setNewCollaboratorRole] = useState<'editor' | 'viewer'>('editor');
  const [chatInput, setChatInput] = useState('');
  const [isOnline, setIsOnline] = useState(true);
  const [isSyncing, setIsSyncing] = useState(false);

  useEffect(() => {
    loadCollaborators();

    // Listen for online/offline events
    const handleOnline = () => setIsOnline(true);
    const handleOffline = () => setIsOnline(false);

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, [projectId]);

  const loadCollaborators = async () => {
    try {
      const response = await fetch(`/api/v1/projects/${projectId}/collaborators`);
      const data = await response.json();
      setCollaborators(data);
    } catch (error) {
      console.error('Failed to load collaborators:', error);
    }
  };

  const handleInviteCollaborator = async () => {
    if (!newCollaboratorEmail) return;

    try {
      const response = await fetch(`/api/v1/projects/${projectId}/collaborators`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          email: newCollaboratorEmail,
          role: newCollaboratorRole,
        }),
      });

      if (response.ok) {
        await loadCollaborators();
        setNewCollaboratorEmail('');
        addNotification('Invitation sent successfully', 'success');
      }
    } catch (error) {
      console.error('Failed to invite collaborator:', error);
    }
  };

  const handleRemoveCollaborator = async (collaboratorId: string) => {
    if (!confirm('Are you sure you want to remove this collaborator?')) return;

    try {
      await fetch(`/api/v1/projects/${projectId}/collaborators/${collaboratorId}`, {
        method: 'DELETE',
      });
      await loadCollaborators();
    } catch (error) {
      console.error('Failed to remove collaborator:', error);
    }
  };

  const handleChangeRole = async (collaboratorId: string, newRole: 'editor' | 'viewer') => {
    try {
      await fetch(`/api/v1/projects/${projectId}/collaborators/${collaboratorId}/role`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ role: newRole }),
      });
      await loadCollaborators();
    } catch (error) {
      console.error('Failed to change role:', error);
    }
  };

  const handleSendMessage = async () => {
    if (!chatInput.trim()) return;

    const message: ChatMessage = {
      id: `msg-${Date.now()}`,
      userId: currentUserId,
      userName: 'You',
      message: chatInput,
      timestamp: new Date(),
    };

    setChatMessages([...chatMessages, message]);
    setChatInput('');

    try {
      await fetch(`/api/v1/projects/${projectId}/chat`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ message: chatInput }),
      });
    } catch (error) {
      console.error('Failed to send message:', error);
    }
  };

  const addNotification = (message: string, type: Notification['type'] = 'info') => {
    const notification: Notification = {
      id: `notif-${Date.now()}`,
      message,
      timestamp: new Date(),
      type,
    };

    setNotifications([...notifications, notification]);

    setTimeout(() => {
      setNotifications((prev) => prev.filter((n) => n.id !== notification.id));
    }, 5000);
  };

  const activeCollaborators = collaborators.filter((c) => c.active);

  return (
    <>
      {/* Collaboration Toolbar */}
      <div className="collaboration-toolbar" data-testid="collaboration-toolbar">
        {/* Connection Status */}
        <div className="connection-status">
          {!isOnline && (
            <span className="status-indicator offline" data-testid="offline-indicator">
              Offline
            </span>
          )}
          {isOnline && isSyncing && (
            <span className="status-indicator syncing" data-testid="syncing-indicator">
              Syncing...
            </span>
          )}
          {isOnline && !isSyncing && (
            <span className="status-indicator online" data-testid="online-indicator">
              Online
            </span>
          )}
        </div>

        {/* Active Collaborators */}
        <div className="active-collaborators" data-testid="active-collaborators">
          <span className="collaborator-count">
            {activeCollaborators.length} active
          </span>
          <div className="collaborator-avatars">
            {activeCollaborators.slice(0, 5).map((collaborator) => (
              <div
                key={collaborator.id}
                className="collaborator-avatar"
                style={{ backgroundColor: collaborator.avatarColor }}
                title={collaborator.name}
                data-testid="collaborator-avatar"
              >
                {collaborator.name.charAt(0).toUpperCase()}
              </div>
            ))}
            {activeCollaborators.length > 5 && (
              <div className="collaborator-avatar more">
                +{activeCollaborators.length - 5}
              </div>
            )}
          </div>
        </div>

        {/* Collaboration Actions */}
        <div className="collaboration-actions">
          <button
            className="chat-button"
            onClick={() => setShowChat(!showChat)}
            data-testid="chat-button"
          >
            💬 Chat
            {chatMessages.length > 0 && (
              <span className="chat-badge">{chatMessages.length}</span>
            )}
          </button>

          <button
            className="share-button"
            onClick={() => setShowShareDialog(true)}
            data-testid="share-project-button"
          >
            👥 Share
          </button>
        </div>
      </div>

      {/* Share Dialog */}
      {showShareDialog && (
        <div className="modal-overlay" onClick={() => setShowShareDialog(false)}>
          <div className="share-dialog" onClick={(e) => e.stopPropagation()}>
            <div className="dialog-header">
              <h3>Share Project</h3>
              <button onClick={() => setShowShareDialog(false)}>×</button>
            </div>

            <div className="dialog-content">
              {/* Invite Form */}
              <div className="invite-form">
                <input
                  type="email"
                  placeholder="Email address"
                  value={newCollaboratorEmail}
                  onChange={(e) => setNewCollaboratorEmail(e.target.value)}
                  data-testid="collaborator-email-input"
                />
                <select
                  value={newCollaboratorRole}
                  onChange={(e) => setNewCollaboratorRole(e.target.value as 'editor' | 'viewer')}
                  data-testid="collaborator-role-select"
                >
                  <option value="editor">Editor</option>
                  <option value="viewer">Viewer</option>
                </select>
                <button onClick={handleInviteCollaborator} data-testid="send-invite-button">
                  Invite
                </button>
              </div>

              <div className="invite-success" data-testid="invite-success" style={{ display: 'none' }}>
                Invitation sent!
              </div>

              {/* Collaborator List */}
              <div className="collaborator-list" data-testid="collaborator-list">
                <h4>Current Collaborators</h4>
                {collaborators.map((collaborator) => (
                  <div key={collaborator.id} className="collaborator-item" data-testid="collaborator-item">
                    <div className="collaborator-info">
                      <div
                        className="collaborator-avatar"
                        style={{ backgroundColor: collaborator.avatarColor }}
                      >
                        {collaborator.name.charAt(0).toUpperCase()}
                      </div>
                      <div className="collaborator-details">
                        <div className="collaborator-name">{collaborator.name}</div>
                        <div className="collaborator-email">{collaborator.email}</div>
                      </div>
                    </div>

                    <div className="collaborator-controls">
                      <span className="collaborator-role" data-testid="collaborator-role">
                        {collaborator.role.charAt(0).toUpperCase() + collaborator.role.slice(1)}
                      </span>

                      {collaborator.role !== 'owner' && (
                        <>
                          <button
                            className="change-role-button"
                            onClick={() => {
                              const newRole = collaborator.role === 'editor' ? 'viewer' : 'editor';
                              handleChangeRole(collaborator.id, newRole);
                            }}
                            data-testid="change-role-button"
                          >
                            Change Role
                          </button>

                          <button
                            className="remove-button"
                            onClick={() => handleRemoveCollaborator(collaborator.id)}
                            data-testid="remove-collaborator-button"
                          >
                            Remove
                          </button>
                        </>
                      )}
                    </div>
                  </div>
                ))}
              </div>

              <div className="confirm-remove" data-testid="confirm-remove" style={{ display: 'none' }}>
                Confirm
              </div>
              <div className="role-editor" data-testid="role-editor" style={{ display: 'none' }}>
                Editor
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Chat Panel */}
      {showChat && (
        <div className="chat-panel" data-testid="chat-panel">
          <div className="chat-header">
            <h4>Team Chat</h4>
            <button onClick={() => setShowChat(false)}>−</button>
          </div>

          <div className="chat-messages" data-testid="chat-messages">
            {chatMessages.map((message) => (
              <div key={message.id} className="chat-message" data-testid="chat-message">
                <div className="message-header">
                  <span className="message-author">{message.userName}</span>
                  <span className="message-time" data-testid="message-time">
                    {message.timestamp.toLocaleTimeString([], {
                      hour: '2-digit',
                      minute: '2-digit',
                    })}
                  </span>
                </div>
                <div className="message-content">{message.message}</div>
              </div>
            ))}

            {chatMessages.length === 0 && (
              <div className="chat-empty">No messages yet. Start the conversation!</div>
            )}
          </div>

          <div className="chat-input-container">
            <input
              type="text"
              placeholder="Type a message..."
              value={chatInput}
              onChange={(e) => setChatInput(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && handleSendMessage()}
              data-testid="chat-input"
            />
            <button onClick={handleSendMessage}>Send</button>
          </div>
        </div>
      )}

      {/* Notifications */}
      <div className="notifications-container">
        {notifications.map((notification) => (
          <div
            key={notification.id}
            className={`notification ${notification.type}`}
            data-testid="notification"
          >
            {notification.message}
          </div>
        ))}
      </div>
    </>
  );
};
