/**
 * Root project structure and metadata
 */

import type { TimeSignature } from './common';
import type { CompositionData } from './composition';
import type { SessionData } from './session';
import type { MixingData } from './mixing';
import type { MasteringData } from './mastering';
import type { PracticeData } from './practice';
import type { AIHistoryData } from './ai-history';
import type { CollaborationData } from './collaboration';

/**
 * Root structure of a .maestro file
 */
export interface MaestroProject {
  formatVersion: string;
  project: {
    metadata: ProjectMetadata;
    composition: CompositionData;
    session: SessionData;
    mixing: MixingData;
    mastering: MasteringData;
    practice: PracticeData;
    aiHistory: AIHistoryData;
    collaboration: CollaborationData;
  };
}

/**
 * Project metadata shared across all modes
 */
export interface ProjectMetadata {
  id: string;
  title: string;
  artist?: string;
  album?: string;
  genre?: string;
  tempo: number;
  timeSignature: TimeSignature;
  key: string;
  created: string;
  modified: string;
  duration?: number;
  tags?: string[];
}
