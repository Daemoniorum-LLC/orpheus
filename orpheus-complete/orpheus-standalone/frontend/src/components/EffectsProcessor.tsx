import React, { useState } from 'react';
import './EffectsProcessor.css';

interface EffectParameter {
  name: string;
  value: number;
  min: number;
  max: number;
  step: number;
  unit: string;
  testId: string;
}

interface Effect {
  id: string;
  type: string;
  name: string;
  enabled: boolean;
  parameters: Record<string, number>;
}

interface EffectsProcessorProps {
  effect: Effect;
  onUpdateParameter: (parameterId: string, value: number) => void;
  onToggleBypass: () => void;
  onRemove: () => void;
}

export const EffectsProcessor: React.FC<EffectsProcessorProps> = ({
  effect,
  onUpdateParameter,
  onToggleBypass,
  onRemove,
}) => {
  const getEffectParameters = (): EffectParameter[] => {
    switch (effect.type) {
      case 'eq':
        return [
          {
            name: 'Band 0 Freq',
            value: effect.parameters['band0Freq'] || 100,
            min: 20,
            max: 20000,
            step: 1,
            unit: 'Hz',
            testId: 'eq-band-0-freq',
          },
          {
            name: 'Band 0 Gain',
            value: effect.parameters['band0Gain'] || 0,
            min: -12,
            max: 12,
            step: 0.1,
            unit: 'dB',
            testId: 'eq-band-0-gain',
          },
          {
            name: 'Band 0 Q',
            value: effect.parameters['band0Q'] || 1,
            min: 0.1,
            max: 10,
            step: 0.1,
            unit: '',
            testId: 'eq-band-0-q',
          },
          {
            name: 'Band 1 Freq',
            value: effect.parameters['band1Freq'] || 500,
            min: 20,
            max: 20000,
            step: 1,
            unit: 'Hz',
            testId: 'eq-band-1-freq',
          },
          {
            name: 'Band 1 Gain',
            value: effect.parameters['band1Gain'] || 0,
            min: -12,
            max: 12,
            step: 0.1,
            unit: 'dB',
            testId: 'eq-band-1-gain',
          },
          {
            name: 'Band 1 Q',
            value: effect.parameters['band1Q'] || 1,
            min: 0.1,
            max: 10,
            step: 0.1,
            unit: '',
            testId: 'eq-band-1-q',
          },
          {
            name: 'Band 2 Freq',
            value: effect.parameters['band2Freq'] || 2000,
            min: 20,
            max: 20000,
            step: 1,
            unit: 'Hz',
            testId: 'eq-band-2-freq',
          },
          {
            name: 'Band 2 Gain',
            value: effect.parameters['band2Gain'] || 0,
            min: -12,
            max: 12,
            step: 0.1,
            unit: 'dB',
            testId: 'eq-band-2-gain',
          },
          {
            name: 'Band 2 Q',
            value: effect.parameters['band2Q'] || 1,
            min: 0.1,
            max: 10,
            step: 0.1,
            unit: '',
            testId: 'eq-band-2-q',
          },
          {
            name: 'Band 3 Freq',
            value: effect.parameters['band3Freq'] || 8000,
            min: 20,
            max: 20000,
            step: 1,
            unit: 'Hz',
            testId: 'eq-band-3-freq',
          },
          {
            name: 'Band 3 Gain',
            value: effect.parameters['band3Gain'] || 0,
            min: -12,
            max: 12,
            step: 0.1,
            unit: 'dB',
            testId: 'eq-band-3-gain',
          },
          {
            name: 'Band 3 Q',
            value: effect.parameters['band3Q'] || 1,
            min: 0.1,
            max: 10,
            step: 0.1,
            unit: '',
            testId: 'eq-band-3-q',
          },
        ];

      case 'compressor':
        return [
          {
            name: 'Threshold',
            value: effect.parameters['threshold'] || -20,
            min: -60,
            max: 0,
            step: 0.1,
            unit: 'dB',
            testId: 'compressor-threshold',
          },
          {
            name: 'Ratio',
            value: effect.parameters['ratio'] || 4,
            min: 1,
            max: 20,
            step: 0.1,
            unit: ':1',
            testId: 'compressor-ratio',
          },
          {
            name: 'Attack',
            value: effect.parameters['attack'] || 10,
            min: 0.1,
            max: 100,
            step: 0.1,
            unit: 'ms',
            testId: 'compressor-attack',
          },
          {
            name: 'Release',
            value: effect.parameters['release'] || 100,
            min: 10,
            max: 1000,
            step: 1,
            unit: 'ms',
            testId: 'compressor-release',
          },
          {
            name: 'Makeup Gain',
            value: effect.parameters['makeupGain'] || 0,
            min: 0,
            max: 24,
            step: 0.1,
            unit: 'dB',
            testId: 'compressor-makeup',
          },
        ];

      case 'reverb':
        return [
          {
            name: 'Room Size',
            value: effect.parameters['roomSize'] || 0.5,
            min: 0,
            max: 1,
            step: 0.01,
            unit: '',
            testId: 'reverb-room-size',
          },
          {
            name: 'Damping',
            value: effect.parameters['damping'] || 0.5,
            min: 0,
            max: 1,
            step: 0.01,
            unit: '',
            testId: 'reverb-damping',
          },
          {
            name: 'Wet Level',
            value: effect.parameters['wet'] || 0.3,
            min: 0,
            max: 1,
            step: 0.01,
            unit: '',
            testId: 'reverb-wet',
          },
          {
            name: 'Dry Level',
            value: effect.parameters['dry'] || 0.7,
            min: 0,
            max: 1,
            step: 0.01,
            unit: '',
            testId: 'reverb-dry',
          },
          {
            name: 'Pre-delay',
            value: effect.parameters['predelay'] || 0,
            min: 0,
            max: 100,
            step: 1,
            unit: 'ms',
            testId: 'reverb-predelay',
          },
        ];

      case 'delay':
        return [
          {
            name: 'Delay Time',
            value: effect.parameters['delayTime'] || 500,
            min: 1,
            max: 2000,
            step: 1,
            unit: 'ms',
            testId: 'delay-time',
          },
          {
            name: 'Feedback',
            value: effect.parameters['feedback'] || 0.3,
            min: 0,
            max: 0.95,
            step: 0.01,
            unit: '',
            testId: 'delay-feedback',
          },
          {
            name: 'Wet Level',
            value: effect.parameters['wet'] || 0.3,
            min: 0,
            max: 1,
            step: 0.01,
            unit: '',
            testId: 'delay-wet',
          },
          {
            name: 'Dry Level',
            value: effect.parameters['dry'] || 0.7,
            min: 0,
            max: 1,
            step: 0.01,
            unit: '',
            testId: 'delay-dry',
          },
        ];

      case 'distortion':
        return [
          {
            name: 'Drive',
            value: effect.parameters['drive'] || 5,
            min: 0,
            max: 100,
            step: 0.1,
            unit: '',
            testId: 'distortion-drive',
          },
          {
            name: 'Tone',
            value: effect.parameters['tone'] || 0.5,
            min: 0,
            max: 1,
            step: 0.01,
            unit: '',
            testId: 'distortion-tone',
          },
          {
            name: 'Output',
            value: effect.parameters['output'] || 0,
            min: -12,
            max: 12,
            step: 0.1,
            unit: 'dB',
            testId: 'distortion-output',
          },
        ];

      case 'limiter':
        return [
          {
            name: 'Threshold',
            value: effect.parameters['threshold'] || -1,
            min: -20,
            max: 0,
            step: 0.1,
            unit: 'dB',
            testId: 'limiter-threshold',
          },
          {
            name: 'Release',
            value: effect.parameters['release'] || 50,
            min: 1,
            max: 1000,
            step: 1,
            unit: 'ms',
            testId: 'limiter-release',
          },
        ];

      default:
        return [];
    }
  };

  const parameters = getEffectParameters();

  return (
    <div className={`effects-processor ${effect.type}`} data-testid="effects-processor">
      <div className="processor-header">
        <h4>{effect.name}</h4>
        <div className="processor-controls">
          <button
            className={`bypass-btn ${!effect.enabled ? 'active' : ''}`}
            onClick={onToggleBypass}
            data-testid="effect-bypass"
          >
            {effect.enabled ? 'Bypass' : 'Bypassed'}
          </button>
          <button className="remove-btn" onClick={onRemove} data-testid="remove-effect">
            Remove
          </button>
        </div>
      </div>

      <div className="processor-parameters">
        {parameters.map((param) => (
          <div key={param.testId} className="parameter-control">
            <label>{param.name}</label>
            <div className="parameter-input">
              <input
                type="range"
                min={param.min}
                max={param.max}
                step={param.step}
                value={param.value}
                onChange={(e) => onUpdateParameter(param.testId, parseFloat(e.target.value))}
                data-testid={param.testId}
              />
              <input
                type="number"
                min={param.min}
                max={param.max}
                step={param.step}
                value={param.value.toFixed(param.step < 1 ? 1 : 0)}
                onChange={(e) => onUpdateParameter(param.testId, parseFloat(e.target.value))}
                data-testid={`${param.testId}-input`}
              />
              <span className="parameter-unit">{param.unit}</span>
            </div>
          </div>
        ))}
      </div>

      {effect.type === 'eq' && (
        <div className="eq-visualizer" data-testid="eq-visualizer">
          <svg viewBox="0 0 400 200" className="eq-graph">
            {/* Grid lines */}
            <line x1="0" y1="100" x2="400" y2="100" stroke="#3a3a3a" strokeWidth="1" />
            <line x1="0" y1="50" x2="400" y2="50" stroke="#2a2a2a" strokeWidth="1" />
            <line x1="0" y1="150" x2="400" y2="150" stroke="#2a2a2a" strokeWidth="1" />

            {/* EQ curve visualization */}
            <path
              d="M 0,100 Q 100,80 200,100 T 400,100"
              fill="none"
              stroke="#3b82f6"
              strokeWidth="2"
            />
          </svg>
        </div>
      )}

      {effect.type === 'compressor' && (
        <div className="compressor-meter" data-testid="gain-reduction-meter">
          <div className="meter-label">Gain Reduction</div>
          <div className="meter-bar">
            <div className="meter-fill" style={{ width: '30%' }} />
          </div>
          <div className="meter-value">-3.2 dB</div>
        </div>
      )}
    </div>
  );
};
