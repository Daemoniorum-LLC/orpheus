/**
 * Tuplet Service - Handles nested tuplet creation, editing, and calculations
 * Supports Guitar Pro 8-style nested tuplets (tuplets within tuplets)
 */

import type { Beat, TupletInfo } from '@orpheus/shared-types';

/**
 * Common tuplet ratios
 */
export const TUPLET_PRESETS = {
  triplet: { actual: 3, normal: 2, name: 'Triplet (3:2)' },
  duplet: { actual: 2, normal: 3, name: 'Duplet (2:3)' },
  quintuplet: { actual: 5, normal: 4, name: 'Quintuplet (5:4)' },
  sextuplet: { actual: 6, normal: 4, name: 'Sextuplet (6:4)' },
  septuplet: { actual: 7, normal: 4, name: 'Septuplet (7:4)' },
  quintupletCompound: { actual: 5, normal: 3, name: 'Quintuplet (5:3)' },
  sextupletCompound: { actual: 6, normal: 3, name: 'Sextuplet (6:3)' },
} as const;

/**
 * Generate a unique tuplet group ID
 */
export function generateTupletGroupId(): string {
  return `tuplet-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
}

/**
 * Calculate the actual duration of a beat within a tuplet
 */
export function calculateTupletDuration(baseDuration: number, tuplet: TupletInfo): number {
  let duration = baseDuration * (tuplet.normal / tuplet.actual);

  // Handle nested tuplets recursively
  if (tuplet.nested) {
    duration = calculateTupletDuration(duration, tuplet.nested);
  }

  return duration;
}

/**
 * Calculate the cumulative tuplet ratio for nested tuplets
 */
export function getCumulativeTupletRatio(tuplet: TupletInfo): { actual: number; normal: number } {
  let actualProduct = tuplet.actual;
  let normalProduct = tuplet.normal;

  let current: TupletInfo | undefined = tuplet.nested;
  while (current) {
    actualProduct *= current.actual;
    normalProduct *= current.normal;
    current = current.nested;
  }

  // Simplify the ratio
  const gcd = greatestCommonDivisor(actualProduct, normalProduct);
  return {
    actual: actualProduct / gcd,
    normal: normalProduct / gcd,
  };
}

/**
 * Calculate greatest common divisor (for simplifying ratios)
 */
function greatestCommonDivisor(a: number, b: number): number {
  return b === 0 ? a : greatestCommonDivisor(b, a % b);
}

/**
 * Get the depth of nested tuplets
 */
export function getTupletDepth(tuplet: TupletInfo): number {
  let depth = 0;
  let current: TupletInfo | undefined = tuplet;

  while (current) {
    depth++;
    current = current.nested;
  }

  return depth;
}

/**
 * Create a simple tuplet
 */
export function createTuplet(
  actual: number,
  normal: number,
  options?: {
    showNumber?: boolean;
    showBracket?: boolean;
    groupId?: string;
  }
): TupletInfo {
  return {
    actual,
    normal,
    showNumber: options?.showNumber ?? true,
    showBracket: options?.showBracket ?? true,
    depth: 0,
    groupId: options?.groupId || generateTupletGroupId(),
  };
}

/**
 * Create a nested tuplet (tuplet within a tuplet)
 */
export function createNestedTuplet(
  parentTuplet: TupletInfo,
  childActual: number,
  childNormal: number
): TupletInfo {
  const nestedTuplet: TupletInfo = {
    actual: childActual,
    normal: childNormal,
    showNumber: true,
    showBracket: true,
    depth: (parentTuplet.depth ?? 0) + 1,
    groupId: generateTupletGroupId(),
  };

  return {
    ...parentTuplet,
    nested: nestedTuplet,
  };
}

/**
 * Add a nested tuplet to an existing beat's tuplet structure
 */
export function addNestedTupletToBeat(beat: Beat, childActual: number, childNormal: number): Beat {
  if (!beat.tuplet) {
    // If no tuplet exists, create a new one
    return {
      ...beat,
      tuplet: createTuplet(childActual, childNormal),
    };
  }

  // Add nested tuplet
  return {
    ...beat,
    tuplet: createNestedTuplet(beat.tuplet, childActual, childNormal),
  };
}

/**
 * Remove the innermost nested tuplet from a beat
 */
export function removeInnermostTuplet(beat: Beat): Beat {
  if (!beat.tuplet) {
    return beat;
  }

  if (!beat.tuplet.nested) {
    // Remove the entire tuplet
    return {
      ...beat,
      tuplet: undefined,
    };
  }

  // Recursively remove the innermost nested tuplet
  const removeNested = (tuplet: TupletInfo): TupletInfo | undefined => {
    if (!tuplet.nested) {
      return undefined;
    }
    if (!tuplet.nested.nested) {
      // This nested tuplet is the innermost, remove it
      return {
        ...tuplet,
        nested: undefined,
      };
    }
    // Continue recursively
    return {
      ...tuplet,
      nested: removeNested(tuplet.nested),
    };
  };

  return {
    ...beat,
    tuplet: removeNested(beat.tuplet),
  };
}

/**
 * Apply tuplet to a group of beats
 */
export function applyTupletToBeats(
  beats: Beat[],
  startIndex: number,
  count: number,
  actual: number,
  normal: number
): Beat[] {
  const groupId = generateTupletGroupId();
  const result = [...beats];

  for (let i = 0; i < count && startIndex + i < beats.length; i++) {
    const index = startIndex + i;
    const bracket: TupletInfo['bracket'] =
      i === 0 ? 'start' : i === count - 1 ? 'stop' : 'continue';

    result[index] = {
      ...result[index],
      tuplet: {
        actual,
        normal,
        bracket,
        showNumber: i === 0, // Only show number on first beat
        showBracket: true,
        depth: 0,
        groupId,
      },
    };
  }

  return result;
}

/**
 * Convert tuplet to alphaTab TEX notation
 */
export function tupletToTEX(tuplet: TupletInfo): string {
  // alphaTab TEX format: :n{duration} for tuplets
  // For example, triplet eighth notes: :8{3}
  // Nested tuplets are represented with additional modifiers

  let tex = `:${tuplet.actual}`;

  if (tuplet.nested) {
    // For nested tuplets, we need to express the compound ratio
    const ratio = getCumulativeTupletRatio(tuplet);
    tex = `{${ratio.actual}:${ratio.normal}}`;
  }

  return tex;
}

/**
 * Parse tuplet from alphaTab TEX notation
 */
export function parseTupletFromTEX(tex: string): TupletInfo | undefined {
  // Match patterns like :3, :5, {3:2}, {5:4:3} for nested
  const simpleMatch = tex.match(/:(\d+)/);
  if (simpleMatch) {
    const actual = parseInt(simpleMatch[1], 10);
    // Common tuplet mappings
    const normalMap: Record<number, number> = {
      3: 2, // triplet
      5: 4, // quintuplet
      6: 4, // sextuplet
      7: 4, // septuplet
      9: 8, // nonuplet
    };
    return {
      actual,
      normal: normalMap[actual] || actual - 1,
    };
  }

  const compoundMatch = tex.match(/\{(\d+):(\d+)\}/);
  if (compoundMatch) {
    return {
      actual: parseInt(compoundMatch[1], 10),
      normal: parseInt(compoundMatch[2], 10),
    };
  }

  return undefined;
}

/**
 * Validate tuplet structure
 */
export function validateTuplet(tuplet: TupletInfo): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  if (tuplet.actual < 2) {
    errors.push('Actual notes must be at least 2');
  }

  if (tuplet.normal < 1) {
    errors.push('Normal notes must be at least 1');
  }

  if (tuplet.actual === tuplet.normal) {
    errors.push('Actual and normal values cannot be equal (not a tuplet)');
  }

  // Check nested depth doesn't exceed reasonable limits
  const depth = getTupletDepth(tuplet);
  if (depth > 3) {
    errors.push('Nested tuplet depth cannot exceed 3 levels');
  }

  // Validate nested tuplets recursively
  if (tuplet.nested) {
    const nestedValidation = validateTuplet(tuplet.nested);
    errors.push(...nestedValidation.errors.map((e) => `Nested: ${e}`));
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Get display string for tuplet (e.g., "3:2" or "3:2 > 3:2" for nested)
 */
export function getTupletDisplayString(tuplet: TupletInfo): string {
  let display = `${tuplet.actual}:${tuplet.normal}`;

  if (tuplet.nested) {
    display += ` > ${getTupletDisplayString(tuplet.nested)}`;
  }

  return display;
}

/**
 * Get preset name if tuplet matches a known preset
 */
export function getTupletPresetName(tuplet: TupletInfo): string | undefined {
  for (const [key, preset] of Object.entries(TUPLET_PRESETS)) {
    if (preset.actual === tuplet.actual && preset.normal === tuplet.normal) {
      return preset.name;
    }
  }
  return undefined;
}

/**
 * Check if two tuplets are equivalent
 */
export function areTupletsEquivalent(a: TupletInfo | undefined, b: TupletInfo | undefined): boolean {
  if (!a && !b) return true;
  if (!a || !b) return false;

  if (a.actual !== b.actual || a.normal !== b.normal) {
    return false;
  }

  return areTupletsEquivalent(a.nested, b.nested);
}

/**
 * Merge consecutive tuplet groups into one
 */
export function mergeTupletGroups(beats: Beat[], groupIds: string[]): Beat[] {
  if (groupIds.length < 2) return beats;

  const newGroupId = groupIds[0];
  const result = beats.map((beat) => {
    if (beat.tuplet && groupIds.includes(beat.tuplet.groupId || '')) {
      return {
        ...beat,
        tuplet: {
          ...beat.tuplet,
          groupId: newGroupId,
        },
      };
    }
    return beat;
  });

  // Update brackets
  const groupBeats = result.filter(
    (b) => b.tuplet?.groupId === newGroupId
  );

  return result.map((beat) => {
    if (beat.tuplet?.groupId === newGroupId) {
      const index = groupBeats.indexOf(beat);
      return {
        ...beat,
        tuplet: {
          ...beat.tuplet,
          bracket:
            index === 0
              ? 'start'
              : index === groupBeats.length - 1
              ? 'stop'
              : 'continue',
          showNumber: index === 0,
        },
      };
    }
    return beat;
  });
}

/**
 * Split a tuplet group at a specific beat
 */
export function splitTupletGroup(beats: Beat[], atBeatIndex: number): Beat[] {
  const beat = beats[atBeatIndex];
  if (!beat.tuplet?.groupId) return beats;

  const oldGroupId = beat.tuplet.groupId;
  const newGroupId = generateTupletGroupId();
  let foundSplit = false;

  const result = beats.map((b, i) => {
    if (b.tuplet?.groupId !== oldGroupId) return b;

    if (i === atBeatIndex) {
      foundSplit = true;
      return {
        ...b,
        tuplet: {
          ...b.tuplet,
          groupId: newGroupId,
          bracket: 'start' as const,
          showNumber: true,
        },
      };
    }

    if (foundSplit) {
      return {
        ...b,
        tuplet: {
          ...b.tuplet,
          groupId: newGroupId,
        },
      };
    }

    return b;
  });

  // Update bracket endings
  return updateTupletBrackets(result);
}

/**
 * Update tuplet brackets for proper display
 */
function updateTupletBrackets(beats: Beat[]): Beat[] {
  const groupMap = new Map<string, number[]>();

  // Collect beat indices by group
  beats.forEach((beat, index) => {
    if (beat.tuplet?.groupId) {
      const indices = groupMap.get(beat.tuplet.groupId) || [];
      indices.push(index);
      groupMap.set(beat.tuplet.groupId, indices);
    }
  });

  // Update brackets
  return beats.map((beat, index) => {
    if (!beat.tuplet?.groupId) return beat;

    const groupIndices = groupMap.get(beat.tuplet.groupId) || [];
    const positionInGroup = groupIndices.indexOf(index);

    let bracket: TupletInfo['bracket'];
    if (groupIndices.length === 1) {
      bracket = 'none';
    } else if (positionInGroup === 0) {
      bracket = 'start';
    } else if (positionInGroup === groupIndices.length - 1) {
      bracket = 'stop';
    } else {
      bracket = 'continue';
    }

    return {
      ...beat,
      tuplet: {
        ...beat.tuplet,
        bracket,
        showNumber: positionInGroup === 0,
      },
    };
  });
}
