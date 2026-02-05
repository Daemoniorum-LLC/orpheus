/**
 * Automation Lane Component
 * Visual automation lane with editable breakpoints for volume, pan, and effects
 */

import { makeStyles, shorthands, tokens, Select, Button } from '@fluentui/react-components';
import { Add16Regular, Delete16Regular } from '@fluentui/react-icons';
import { useState, useRef, useCallback, useEffect } from 'react';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('4px'),
    backgroundColor: tokens.colorNeutralBackground2,
    ...shorthands.borderRadius('4px'),
    ...shorthands.padding('8px'),
    minHeight: '80px',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    ...shorthands.gap('8px'),
  },
  headerLeft: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
  },
  trackName: {
    fontSize: '11px',
    fontWeight: tokens.fontWeightSemibold,
    color: tokens.colorNeutralForeground2,
    minWidth: '60px',
    ...shorthands.overflow('hidden'),
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  parameterSelect: {
    minWidth: '100px',
  },
  laneContainer: {
    position: 'relative',
    height: '60px',
    backgroundColor: tokens.colorNeutralBackground4,
    ...shorthands.borderRadius('2px'),
    cursor: 'crosshair',
    ...shorthands.overflow('hidden'),
  },
  lanePath: {
    position: 'absolute',
    top: 0,
    left: 0,
    width: '100%',
    height: '100%',
    pointerEvents: 'none',
  },
  point: {
    position: 'absolute',
    width: '10px',
    height: '10px',
    ...shorthands.borderRadius('50%'),
    backgroundColor: tokens.colorBrandBackground,
    ...shorthands.border('2px', 'solid', tokens.colorBrandForeground1),
    transform: 'translate(-50%, -50%)',
    cursor: 'pointer',
    zIndex: 10,
    transition: 'transform 0.1s ease',
    ':hover': {
      transform: 'translate(-50%, -50%) scale(1.3)',
    },
  },
  pointSelected: {
    backgroundColor: tokens.colorPaletteYellowBackground3,
    ...shorthands.border('2px', 'solid', tokens.colorPaletteYellowForeground1),
    transform: 'translate(-50%, -50%) scale(1.2)',
  },
  gridLines: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    pointerEvents: 'none',
  },
  gridLine: {
    position: 'absolute',
    left: 0,
    right: 0,
    height: '1px',
    backgroundColor: tokens.colorNeutralStroke2,
    opacity: 0.3,
  },
  valueLabel: {
    position: 'absolute',
    right: '4px',
    fontSize: '9px',
    color: tokens.colorNeutralForeground3,
    fontFamily: 'monospace',
  },
  noData: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100%',
    color: tokens.colorNeutralForeground3,
    fontSize: '11px',
  },
});

export interface AutomationPoint {
  id: string;
  time: number; // 0-1 normalized position
  value: number; // 0-1 normalized value
  curve?: 'linear' | 'exponential' | 'hold';
}

export type AutomationParameter = 'volume' | 'pan' | 'mute' | 'eq-low' | 'eq-mid' | 'eq-high' | 'reverb' | 'delay';

export interface AutomationLaneProps {
  trackId: string;
  trackName: string;
  parameter?: AutomationParameter;
  points?: AutomationPoint[];
  /** Duration in seconds (for display purposes) */
  duration?: number;
  /** Current playback position (0-1) */
  playheadPosition?: number;
  onPointsChange?: (points: AutomationPoint[]) => void;
  onParameterChange?: (parameter: AutomationParameter) => void;
}

const PARAMETER_OPTIONS: { value: AutomationParameter; label: string }[] = [
  { value: 'volume', label: 'Volume' },
  { value: 'pan', label: 'Pan' },
  { value: 'mute', label: 'Mute' },
  { value: 'eq-low', label: 'EQ Low' },
  { value: 'eq-mid', label: 'EQ Mid' },
  { value: 'eq-high', label: 'EQ High' },
  { value: 'reverb', label: 'Reverb' },
  { value: 'delay', label: 'Delay' },
];

function getParameterRange(param: AutomationParameter): { min: number; max: number; unit: string } {
  switch (param) {
    case 'volume':
      return { min: -60, max: 12, unit: 'dB' };
    case 'pan':
      return { min: -50, max: 50, unit: '' };
    case 'mute':
      return { min: 0, max: 1, unit: '' };
    case 'eq-low':
    case 'eq-mid':
    case 'eq-high':
      return { min: -12, max: 12, unit: 'dB' };
    case 'reverb':
    case 'delay':
      return { min: 0, max: 100, unit: '%' };
    default:
      return { min: 0, max: 1, unit: '' };
  }
}

function normalizedToValue(normalized: number, param: AutomationParameter): number {
  const range = getParameterRange(param);
  return range.min + normalized * (range.max - range.min);
}

function valueToNormalized(value: number, param: AutomationParameter): number {
  const range = getParameterRange(param);
  return (value - range.min) / (range.max - range.min);
}

function formatValue(normalized: number, param: AutomationParameter): string {
  const value = normalizedToValue(normalized, param);
  const range = getParameterRange(param);

  if (param === 'mute') {
    return normalized > 0.5 ? 'ON' : 'OFF';
  }
  if (param === 'pan') {
    if (Math.abs(value) < 1) return 'C';
    return value > 0 ? `R${Math.round(value)}` : `L${Math.abs(Math.round(value))}`;
  }
  return `${value.toFixed(1)}${range.unit}`;
}

export function AutomationLane({
  trackId,
  trackName,
  parameter = 'volume',
  points = [],
  duration = 180,
  playheadPosition = 0,
  onPointsChange,
  onParameterChange,
}: AutomationLaneProps) {
  const styles = useStyles();
  const containerRef = useRef<HTMLDivElement>(null);
  const [selectedPointId, setSelectedPointId] = useState<string | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [isFocused, setIsFocused] = useState(false);
  const [localPoints, setLocalPoints] = useState<AutomationPoint[]>(points);

  // Sync local points with prop
  useEffect(() => {
    setLocalPoints(points);
  }, [points]);

  // Get position from mouse or touch event
  const getPositionFromEvent = useCallback((e: React.MouseEvent | MouseEvent | React.TouchEvent | TouchEvent): { time: number; value: number } => {
    if (!containerRef.current) return { time: 0, value: 0 };

    const rect = containerRef.current.getBoundingClientRect();
    let clientX: number, clientY: number;

    if ('touches' in e) {
      // Touch event
      const touch = e.touches[0] || (e as TouchEvent).changedTouches[0];
      clientX = touch.clientX;
      clientY = touch.clientY;
    } else {
      // Mouse event
      clientX = e.clientX;
      clientY = e.clientY;
    }

    const x = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    const y = Math.max(0, Math.min(1, 1 - (clientY - rect.top) / rect.height));

    return { time: x, value: y };
  }, []);

  const addPoint = useCallback((time: number, value: number) => {
    const newPoint: AutomationPoint = {
      id: `point-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      time,
      value,
      curve: 'linear',
    };

    const newPoints = [...localPoints, newPoint].sort((a, b) => a.time - b.time);
    setLocalPoints(newPoints);
    onPointsChange?.(newPoints);
    setSelectedPointId(newPoint.id);
  }, [localPoints, onPointsChange]);

  const updatePoint = useCallback((id: string, updates: Partial<AutomationPoint>) => {
    const newPoints = localPoints.map((p) =>
      p.id === id ? { ...p, ...updates } : p
    ).sort((a, b) => a.time - b.time);

    setLocalPoints(newPoints);
    onPointsChange?.(newPoints);
  }, [localPoints, onPointsChange]);

  const deletePoint = useCallback((id: string) => {
    const newPoints = localPoints.filter((p) => p.id !== id);
    setLocalPoints(newPoints);
    onPointsChange?.(newPoints);
    if (selectedPointId === id) {
      setSelectedPointId(null);
    }
  }, [localPoints, onPointsChange, selectedPointId]);

  const handleLaneClick = useCallback((e: React.MouseEvent) => {
    if (isDragging) return;

    const { time, value } = getPositionFromEvent(e);
    addPoint(time, value);
  }, [isDragging, getPositionFromEvent, addPoint]);

  // Mouse drag handling
  const handlePointMouseDown = useCallback((e: React.MouseEvent, pointId: string) => {
    e.stopPropagation();
    setSelectedPointId(pointId);
    setIsDragging(true);

    const handleMouseMove = (moveEvent: MouseEvent) => {
      const { time, value } = getPositionFromEvent(moveEvent);
      updatePoint(pointId, { time, value });
    };

    const handleMouseUp = () => {
      setIsDragging(false);
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }, [getPositionFromEvent, updatePoint]);

  // Touch drag handling
  const handlePointTouchStart = useCallback((e: React.TouchEvent, pointId: string) => {
    e.stopPropagation();
    setSelectedPointId(pointId);
    setIsDragging(true);

    const handleTouchMove = (moveEvent: TouchEvent) => {
      moveEvent.preventDefault();
      const { time, value } = getPositionFromEvent(moveEvent);
      updatePoint(pointId, { time, value });
    };

    const handleTouchEnd = () => {
      setIsDragging(false);
      document.removeEventListener('touchmove', handleTouchMove);
      document.removeEventListener('touchend', handleTouchEnd);
    };

    document.addEventListener('touchmove', handleTouchMove, { passive: false });
    document.addEventListener('touchend', handleTouchEnd);
  }, [getPositionFromEvent, updatePoint]);

  const handleDeleteSelected = useCallback(() => {
    if (selectedPointId) {
      deletePoint(selectedPointId);
    }
  }, [selectedPointId, deletePoint]);

  // Handle keyboard shortcuts - only when focused
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Delete' || e.key === 'Backspace') {
      if (selectedPointId) {
        e.preventDefault();
        handleDeleteSelected();
      }
    }
    // Arrow keys for fine adjustment
    if (selectedPointId && (e.key === 'ArrowUp' || e.key === 'ArrowDown' || e.key === 'ArrowLeft' || e.key === 'ArrowRight')) {
      e.preventDefault();
      const point = localPoints.find(p => p.id === selectedPointId);
      if (point) {
        const step = e.shiftKey ? 0.1 : 0.01;
        let newTime = point.time;
        let newValue = point.value;

        if (e.key === 'ArrowUp') newValue = Math.min(1, point.value + step);
        if (e.key === 'ArrowDown') newValue = Math.max(0, point.value - step);
        if (e.key === 'ArrowRight') newTime = Math.min(1, point.time + step);
        if (e.key === 'ArrowLeft') newTime = Math.max(0, point.time - step);

        updatePoint(selectedPointId, { time: newTime, value: newValue });
      }
    }
    // Escape to deselect
    if (e.key === 'Escape') {
      setSelectedPointId(null);
    }
  }, [selectedPointId, handleDeleteSelected, localPoints, updatePoint]);

  // Generate SVG path for automation curve
  const generatePath = useCallback(() => {
    if (localPoints.length === 0) return '';

    const sortedPoints = [...localPoints].sort((a, b) => a.time - b.time);
    const pathParts: string[] = [];

    // Start from left edge at first point's value or default
    const startY = sortedPoints.length > 0 ? (1 - sortedPoints[0].value) * 100 : 50;
    pathParts.push(`M 0 ${startY}%`);

    for (let i = 0; i < sortedPoints.length; i++) {
      const point = sortedPoints[i];
      const x = point.time * 100;
      const y = (1 - point.value) * 100;

      if (i === 0 && point.time > 0) {
        // Line from left edge to first point
        pathParts.push(`L ${x}% ${y}%`);
      } else if (i > 0) {
        const prevPoint = sortedPoints[i - 1];

        if (prevPoint.curve === 'hold') {
          // Step function - hold previous value then jump
          pathParts.push(`L ${x}% ${(1 - prevPoint.value) * 100}%`);
          pathParts.push(`L ${x}% ${y}%`);
        } else {
          // Linear interpolation
          pathParts.push(`L ${x}% ${y}%`);
        }
      }
    }

    // Continue to right edge
    const lastPoint = sortedPoints[sortedPoints.length - 1];
    if (lastPoint && lastPoint.time < 1) {
      pathParts.push(`L 100% ${(1 - lastPoint.value) * 100}%`);
    }

    return pathParts.join(' ');
  }, [localPoints]);

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <span className={styles.trackName}>{trackName}</span>
          <Select
            className={styles.parameterSelect}
            size="small"
            value={parameter}
            onChange={(_, data) => onParameterChange?.(data.value as AutomationParameter)}
          >
            {PARAMETER_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </Select>
        </div>
        <div style={{ display: 'flex', gap: '4px' }}>
          <Button
            icon={<Delete16Regular />}
            size="small"
            appearance="subtle"
            disabled={!selectedPointId}
            onClick={handleDeleteSelected}
            title="Delete selected point"
          />
        </div>
      </div>

      <div
        ref={containerRef}
        className={styles.laneContainer}
        onClick={handleLaneClick}
        onKeyDown={handleKeyDown}
        onFocus={() => setIsFocused(true)}
        onBlur={() => setIsFocused(false)}
        tabIndex={0}
        role="application"
        aria-label={`${trackName} ${parameter} automation lane. ${localPoints.length} points. Click to add points, use arrow keys to adjust selected point.`}
        style={{ outline: isFocused ? `2px solid ${tokens.colorBrandForeground1}` : undefined }}
      >
        {/* Grid lines */}
        <div className={styles.gridLines}>
          <div className={styles.gridLine} style={{ top: '25%' }} />
          <div className={styles.gridLine} style={{ top: '50%' }} />
          <div className={styles.gridLine} style={{ top: '75%' }} />
          {/* Value labels */}
          <span className={styles.valueLabel} style={{ top: '0' }}>
            {formatValue(1, parameter)}
          </span>
          <span className={styles.valueLabel} style={{ top: '48%' }}>
            {formatValue(0.5, parameter)}
          </span>
          <span className={styles.valueLabel} style={{ bottom: '0' }}>
            {formatValue(0, parameter)}
          </span>
        </div>

        {/* Automation curve */}
        {localPoints.length > 0 && (
          <svg className={styles.lanePath} viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
            <path
              d={generatePath()}
              fill="none"
              stroke={tokens.colorBrandBackground}
              strokeWidth="2"
              vectorEffect="non-scaling-stroke"
            />
          </svg>
        )}

        {/* Playhead */}
        {playheadPosition > 0 && (
          <div
            style={{
              position: 'absolute',
              left: `${playheadPosition * 100}%`,
              top: 0,
              bottom: 0,
              width: '1px',
              backgroundColor: tokens.colorPaletteRedForeground1,
              pointerEvents: 'none',
              zIndex: 5,
            }}
            aria-hidden="true"
          />
        )}

        {/* Automation points */}
        {localPoints.map((point, index) => (
          <div
            key={point.id}
            className={`${styles.point} ${selectedPointId === point.id ? styles.pointSelected : ''}`}
            style={{
              left: `${point.time * 100}%`,
              top: `${(1 - point.value) * 100}%`,
            }}
            onMouseDown={(e) => handlePointMouseDown(e, point.id)}
            onTouchStart={(e) => handlePointTouchStart(e, point.id)}
            role="slider"
            aria-label={`Point ${index + 1}: ${formatValue(point.value, parameter)} at ${(point.time * duration).toFixed(1)}s`}
            aria-valuenow={point.value * 100}
            aria-valuemin={0}
            aria-valuemax={100}
            tabIndex={-1}
            title={`${formatValue(point.value, parameter)} @ ${(point.time * duration).toFixed(1)}s`}
          />
        ))}

        {localPoints.length === 0 && (
          <div className={styles.noData}>
            Click to add automation points
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * Container for multiple automation lanes
 */
export interface AutomationLanesContainerProps {
  tracks: { id: string; name: string; color?: string }[];
  automationData?: Record<string, Record<AutomationParameter, AutomationPoint[]>>;
  duration?: number;
  playheadPosition?: number;
  onAutomationChange?: (trackId: string, parameter: AutomationParameter, points: AutomationPoint[]) => void;
}

export function AutomationLanesContainer({
  tracks,
  automationData = {},
  duration = 180,
  playheadPosition = 0,
  onAutomationChange,
}: AutomationLanesContainerProps) {
  const [selectedParameters, setSelectedParameters] = useState<Record<string, AutomationParameter>>(
    tracks.reduce((acc, track) => ({ ...acc, [track.id]: 'volume' }), {})
  );

  const handleParameterChange = useCallback((trackId: string, parameter: AutomationParameter) => {
    setSelectedParameters((prev) => ({ ...prev, [trackId]: parameter }));
  }, []);

  const handlePointsChange = useCallback((trackId: string, parameter: AutomationParameter, points: AutomationPoint[]) => {
    onAutomationChange?.(trackId, parameter, points);
  }, [onAutomationChange]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
      {tracks.map((track) => {
        const parameter = selectedParameters[track.id] || 'volume';
        const points = automationData[track.id]?.[parameter] || [];

        return (
          <AutomationLane
            key={track.id}
            trackId={track.id}
            trackName={track.name}
            parameter={parameter}
            points={points}
            duration={duration}
            playheadPosition={playheadPosition}
            onParameterChange={(p) => handleParameterChange(track.id, p)}
            onPointsChange={(pts) => handlePointsChange(track.id, parameter, pts)}
          />
        );
      })}
    </div>
  );
}
