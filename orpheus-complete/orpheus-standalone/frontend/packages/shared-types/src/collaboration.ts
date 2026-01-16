/**
 * Collaboration types for multi-user collaboration
 */

import type { MaestroMode } from './common';

export interface CollaborationData {
  enabled: boolean;
  collaborators: Collaborator[];
  permissions: Record<string, Permission[]>;
  versions: Version[];
  comments: Comment[];
}

export interface Collaborator {
  userId: string;
  name: string;
  email: string;
  role: 'owner' | 'editor' | 'viewer';
  joinedAt: string;
}

export type Permission =
  | 'read'
  | 'write-composition'
  | 'write-session'
  | 'write-mixing'
  | 'export'
  | 'admin';

export interface Version {
  id: string;
  timestamp: string;
  userId: string;
  description?: string;
  changes: Change[];
}

export interface Change {
  path: string;
  operation: 'add' | 'remove' | 'replace';
  oldValue?: any;
  newValue?: any;
}

export interface Comment {
  id: string;
  userId: string;
  timestamp: string;
  context: CommentContext;
  content: string;
  resolved: boolean;
  replies: CommentReply[];
}

export interface CommentContext {
  mode: MaestroMode;
  trackId?: string;
  measureNumber?: number;
  timePosition?: number;
}

export interface CommentReply {
  userId: string;
  timestamp: string;
  content: string;
}
