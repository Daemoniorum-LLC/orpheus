/**
 * Practice types for practice mode and progress tracking
 */

export interface PracticeData {
  speedTrainer: SpeedTrainerSettings;
  loop: LoopSettings;
  practiceLog: PracticeSession[];
  difficultSections: DifficultSection[];
}

export interface SpeedTrainerSettings {
  enabled: boolean;
  currentSpeed: number;
  targetSpeed: number;
  incrementStep: number;
}

export interface LoopSettings {
  enabled: boolean;
  startMeasure: number;
  endMeasure: number;
  repeatCount?: number;
}

export interface PracticeSession {
  id: string;
  date: string;
  duration: number;
  tempo: number;
  sectionsWorked: string[];
  notes?: string;
  analysis?: PerformanceAnalysis;
}

export interface PerformanceAnalysis {
  timingAccuracy: number;
  noteAccuracy: number;
  strengths: string[];
  areasToImprove: string[];
}

export interface DifficultSection {
  sectionName: string;
  measures: [number, number];
  difficulty: 'easy' | 'medium' | 'hard' | 'expert';
  reason: string;
  practiceCount: number;
  lastPracticed?: string;
}
