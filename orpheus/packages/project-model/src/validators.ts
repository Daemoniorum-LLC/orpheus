/**
 * Validators for Maestro project files
 */

export interface ValidationResult {
  valid: boolean;
  errors: string[];
}

/**
 * Validates a Maestro project structure
 */
export function validateProject(project: any): ValidationResult {
  const errors: string[] = [];

  // Check top-level structure
  if (!project || typeof project !== 'object') {
    errors.push('Project must be an object');
    return { valid: false, errors };
  }

  if (!project.formatVersion) {
    errors.push('Missing formatVersion');
  } else if (typeof project.formatVersion !== 'string') {
    errors.push('formatVersion must be a string');
  }

  if (!project.project) {
    errors.push('Missing project data');
    return { valid: false, errors };
  }

  // Validate metadata
  const metadata = project.project.metadata;
  if (!metadata) {
    errors.push('Missing project metadata');
  } else {
    if (!metadata.id || typeof metadata.id !== 'string') {
      errors.push('Invalid or missing project id');
    }
    if (!metadata.title || typeof metadata.title !== 'string') {
      errors.push('Invalid or missing project title');
    }
    if (typeof metadata.tempo !== 'number' || metadata.tempo <= 0) {
      errors.push('Invalid tempo (must be positive number)');
    }
    if (!metadata.created || !isValidISO8601(metadata.created)) {
      errors.push('Invalid created timestamp');
    }
    if (!metadata.modified || !isValidISO8601(metadata.modified)) {
      errors.push('Invalid modified timestamp');
    }
  }

  // Validate required sections exist
  const requiredSections = [
    'composition',
    'session',
    'mixing',
    'mastering',
    'practice',
    'aiHistory',
    'collaboration',
  ];

  for (const section of requiredSections) {
    if (!project.project[section]) {
      errors.push(`Missing ${section} section`);
    }
  }

  // Validate session data
  const session = project.project.session;
  if (session) {
    if (typeof session.sampleRate !== 'number' || session.sampleRate <= 0) {
      errors.push('Invalid sampleRate');
    }
    if (typeof session.bitDepth !== 'number' || ![16, 24, 32].includes(session.bitDepth)) {
      errors.push('Invalid bitDepth (must be 16, 24, or 32)');
    }
    if (!Array.isArray(session.tracks)) {
      errors.push('session.tracks must be an array');
    }
    if (!Array.isArray(session.buses)) {
      errors.push('session.buses must be an array');
    } else {
      // Must have a master bus
      const hasMaster = session.buses.some((bus: any) => bus.type === 'master');
      if (!hasMaster) {
        errors.push('session.buses must include a master bus');
      }
    }
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Validates an ISO 8601 timestamp string
 */
function isValidISO8601(dateString: string): boolean {
  const date = new Date(dateString);
  return date.toISOString() === dateString;
}

/**
 * Validates a score structure
 */
export function validateScore(score: any): ValidationResult {
  const errors: string[] = [];

  if (!score.id || typeof score.id !== 'string') {
    errors.push('Score must have a valid id');
  }
  if (!score.title || typeof score.title !== 'string') {
    errors.push('Score must have a title');
  }
  if (!score.instrument || typeof score.instrument !== 'string') {
    errors.push('Score must have an instrument type');
  }
  if (!Array.isArray(score.tracks)) {
    errors.push('Score must have tracks array');
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Validates a track structure
 */
export function validateTrack(track: any): ValidationResult {
  const errors: string[] = [];

  if (!track.id || typeof track.id !== 'string') {
    errors.push('Track must have a valid id');
  }
  if (!track.name || typeof track.name !== 'string') {
    errors.push('Track must have a name');
  }
  if (!['audio', 'midi', 'instrument', 'aux'].includes(track.type)) {
    errors.push('Track must have a valid type');
  }
  if (!Array.isArray(track.regions)) {
    errors.push('Track must have regions array');
  }
  if (!Array.isArray(track.plugins)) {
    errors.push('Track must have plugins array');
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}
