/**
 * User Preferences Dialog
 * Settings for theme, auto-save, accessibility, and more
 */

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
  Button,
  Label,
  Slider,
} from '@persona-framework/ui';
import { Switch } from './Switch';
import {
  Settings,
  Sun,
  Moon,
  Monitor,
  Save,
  RotateCcw,
  Keyboard,
  Eye,
  Volume2,
  Mic,
} from 'lucide-react';
import { useState, useEffect } from 'react';
import { cn } from '../lib/utils';
import { useTheme } from '../contexts/ThemeContext';
import { useOnboarding } from './OnboardingDialog';
import { safeStorage } from '../utils/storage';

// Preferences storage key
const PREFERENCES_KEY = 'orpheus-preferences';

export interface UserPreferences {
  // Appearance
  theme: 'light' | 'dark' | 'system';
  reducedMotion: boolean;
  highContrast: boolean;

  // Auto-save
  autoSaveEnabled: boolean;
  autoSaveInterval: number; // in seconds

  // Audio
  masterVolume: number;
  metronomeEnabled: boolean;
  metronomeVolume: number;

  // UI
  showTooltips: boolean;
  compactMode: boolean;
  sidebarDefaultOpen: boolean;

  // Voice Commands
  voiceEnabled: boolean;
  voiceWakeWordEnabled: boolean;
  voiceContinuousMode: boolean;
  voiceAudioFeedback: boolean;  // Play audio tones for feedback
}

const defaultPreferences: UserPreferences = {
  theme: 'system',
  reducedMotion: false,
  highContrast: false,
  autoSaveEnabled: true,
  autoSaveInterval: 30,
  masterVolume: 0.8,
  metronomeEnabled: true,
  metronomeVolume: 0.5,
  showTooltips: true,
  compactMode: false,
  sidebarDefaultOpen: true,
  voiceEnabled: true,
  voiceWakeWordEnabled: true,
  voiceContinuousMode: false,
  voiceAudioFeedback: true,
};

// Load preferences from localStorage
export function loadPreferences(): UserPreferences {
  try {
    const stored = safeStorage.getItem(PREFERENCES_KEY);
    if (stored) {
      return { ...defaultPreferences, ...JSON.parse(stored) };
    }
  } catch (e) {
    console.error('Failed to load preferences:', e);
  }
  return defaultPreferences;
}

// Save preferences to localStorage
export function savePreferences(prefs: UserPreferences): void {
  try {
    safeStorage.setItem(PREFERENCES_KEY, JSON.stringify(prefs));
  } catch (e) {
    console.error('Failed to save preferences:', e);
  }
}

interface PreferencesDialogProps {
  open: boolean;
  onClose: () => void;
}

type TabId = 'appearance' | 'behavior' | 'audio' | 'voice' | 'help';

export function PreferencesDialog({ open, onClose }: PreferencesDialogProps) {
  const [activeTab, setActiveTab] = useState<TabId>('appearance');
  const [preferences, setPreferences] = useState<UserPreferences>(loadPreferences);
  const [hasChanges, setHasChanges] = useState(false);
  const { theme, setTheme } = useTheme();
  const { resetOnboarding } = useOnboarding();

  // Load preferences when dialog opens
  useEffect(() => {
    if (open) {
      setPreferences(loadPreferences());
      setHasChanges(false);
    }
  }, [open]);

  const updatePreference = <K extends keyof UserPreferences>(
    key: K,
    value: UserPreferences[K]
  ) => {
    setPreferences((prev) => ({ ...prev, [key]: value }));
    setHasChanges(true);

    // Apply theme changes immediately
    if (key === 'theme') {
      setTheme(value as 'light' | 'dark' | 'system');
    }
  };

  const handleSave = () => {
    savePreferences(preferences);
    setHasChanges(false);
    onClose();
  };

  const handleReset = () => {
    setPreferences(defaultPreferences);
    setHasChanges(true);
  };

  const tabs: { id: TabId; label: string; icon: React.ElementType }[] = [
    { id: 'appearance', label: 'Appearance', icon: Eye },
    { id: 'behavior', label: 'Behavior', icon: Settings },
    { id: 'audio', label: 'Audio', icon: Volume2 },
    { id: 'voice', label: 'Voice', icon: Mic },
    { id: 'help', label: 'Help', icon: Keyboard },
  ];

  return (
    <Dialog open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
      <DialogContent className="max-w-[650px] max-h-[80vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Settings className="h-5 w-5" />
            Preferences
          </DialogTitle>
          <DialogDescription>
            Customize your Orpheus experience
          </DialogDescription>
        </DialogHeader>

        <div className="flex gap-4 flex-1 min-h-0 py-4">
          {/* Sidebar tabs */}
          <nav className="flex flex-col gap-1 w-[140px] flex-shrink-0" role="tablist" aria-label="Preference categories">
            {tabs.map((tab) => {
              const Icon = tab.icon;
              return (
                <button
                  key={tab.id}
                  role="tab"
                  aria-selected={activeTab === tab.id}
                  aria-controls={`${tab.id}-panel`}
                  onClick={() => setActiveTab(tab.id)}
                  className={cn(
                    // min-h-[44px] ensures WCAG touch target minimum
                    'flex items-center gap-2 px-3 py-2.5 min-h-[44px] rounded-lg text-sm text-left transition-colors',
                    activeTab === tab.id
                      ? 'bg-primary text-primary-foreground'
                      : 'hover:bg-muted text-muted-foreground hover:text-foreground'
                  )}
                >
                  <Icon className="h-4 w-4" />
                  {tab.label}
                </button>
              );
            })}
          </nav>

          {/* Content area */}
          <div className="flex-1 overflow-y-auto pr-2">
            {activeTab === 'appearance' && (
              <div role="tabpanel" id="appearance-panel" aria-labelledby="appearance-tab" className="space-y-6">
                <div className="space-y-3">
                  <Label className="text-sm font-medium" id="theme-label">Theme</Label>
                  <div className="grid grid-cols-3 gap-2" role="radiogroup" aria-labelledby="theme-label">
                    {[
                      { value: 'light', icon: Sun, label: 'Light' },
                      { value: 'dark', icon: Moon, label: 'Dark' },
                      { value: 'system', icon: Monitor, label: 'System' },
                    ].map((option) => {
                      const Icon = option.icon;
                      const isSelected = preferences.theme === option.value;
                      return (
                        <button
                          key={option.value}
                          role="radio"
                          aria-checked={isSelected}
                          onClick={() =>
                            updatePreference('theme', option.value as UserPreferences['theme'])
                          }
                          className={cn(
                            // min-h-[60px] ensures adequate touch target
                            'flex flex-col items-center justify-center gap-2 p-3 min-h-[60px] rounded-lg border transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
                            isSelected
                              ? 'border-primary bg-primary/10'
                              : 'border-border hover:border-primary/50'
                          )}
                        >
                          <Icon className="h-5 w-5" />
                          <span className="text-xs">{option.label}</span>
                        </button>
                      );
                    })}
                  </div>
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label className="text-sm font-medium">Reduced Motion</Label>
                    <p className="text-xs text-muted-foreground">
                      Minimize animations for accessibility
                    </p>
                  </div>
                  <Switch
                    checked={preferences.reducedMotion}
                    onCheckedChange={(checked) => updatePreference('reducedMotion', checked)}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label className="text-sm font-medium">High Contrast</Label>
                    <p className="text-xs text-muted-foreground">
                      Increase contrast for better visibility
                    </p>
                  </div>
                  <Switch
                    checked={preferences.highContrast}
                    onCheckedChange={(checked) => updatePreference('highContrast', checked)}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label className="text-sm font-medium">Compact Mode</Label>
                    <p className="text-xs text-muted-foreground">
                      Reduce padding and spacing
                    </p>
                  </div>
                  <Switch
                    checked={preferences.compactMode}
                    onCheckedChange={(checked) => updatePreference('compactMode', checked)}
                  />
                </div>
              </div>
            )}

            {activeTab === 'behavior' && (
              <div role="tabpanel" id="behavior-panel" aria-labelledby="behavior-tab" className="space-y-6">
                <div className="flex items-center justify-between">
                  <div>
                    <Label className="text-sm font-medium">Auto-Save</Label>
                    <p className="text-xs text-muted-foreground">
                      Automatically save your work
                    </p>
                  </div>
                  <Switch
                    checked={preferences.autoSaveEnabled}
                    onCheckedChange={(checked) => updatePreference('autoSaveEnabled', checked)}
                  />
                </div>

                {preferences.autoSaveEnabled && (
                  <div className="space-y-2">
                    <div className="flex items-center justify-between">
                      <Label className="text-sm font-medium" id="autosave-interval-label">Auto-Save Interval</Label>
                      <span className="text-sm font-medium text-primary">
                        {preferences.autoSaveInterval}s
                      </span>
                    </div>
                    <div className="flex items-center gap-3">
                      <span className="text-xs text-muted-foreground w-8">10s</span>
                      <Slider
                        min={10}
                        max={120}
                        step={10}
                        value={[preferences.autoSaveInterval]}
                        onValueChange={(value) => updatePreference('autoSaveInterval', value[0])}
                        aria-labelledby="autosave-interval-label"
                        aria-valuemin={10}
                        aria-valuemax={120}
                        aria-valuenow={preferences.autoSaveInterval}
                        aria-valuetext={`${preferences.autoSaveInterval} seconds`}
                        className="flex-1"
                      />
                      <span className="text-xs text-muted-foreground w-10">120s</span>
                    </div>
                  </div>
                )}

                <div className="flex items-center justify-between">
                  <div>
                    <Label className="text-sm font-medium">Show Tooltips</Label>
                    <p className="text-xs text-muted-foreground">
                      Display helpful tooltips on hover
                    </p>
                  </div>
                  <Switch
                    checked={preferences.showTooltips}
                    onCheckedChange={(checked) => updatePreference('showTooltips', checked)}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label className="text-sm font-medium">Sidebar Open by Default</Label>
                    <p className="text-xs text-muted-foreground">
                      Show sidebar when starting the app
                    </p>
                  </div>
                  <Switch
                    checked={preferences.sidebarDefaultOpen}
                    onCheckedChange={(checked) => updatePreference('sidebarDefaultOpen', checked)}
                  />
                </div>
              </div>
            )}

            {activeTab === 'audio' && (
              <div role="tabpanel" id="audio-panel" aria-labelledby="audio-tab" className="space-y-6">
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <Label className="text-sm font-medium" id="master-volume-label">Master Volume</Label>
                    <span className="text-sm font-medium text-primary">
                      {Math.round(preferences.masterVolume * 100)}%
                    </span>
                  </div>
                  <div className="flex items-center gap-3">
                    <span className="text-xs text-muted-foreground w-6">0%</span>
                    <Slider
                      min={0}
                      max={1}
                      step={0.05}
                      value={[preferences.masterVolume]}
                      onValueChange={(value) => updatePreference('masterVolume', value[0])}
                      aria-labelledby="master-volume-label"
                      aria-valuemin={0}
                      aria-valuemax={100}
                      aria-valuenow={Math.round(preferences.masterVolume * 100)}
                      aria-valuetext={`${Math.round(preferences.masterVolume * 100)} percent`}
                      className="flex-1"
                    />
                    <span className="text-xs text-muted-foreground w-10">100%</span>
                  </div>
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label className="text-sm font-medium">Metronome</Label>
                    <p className="text-xs text-muted-foreground">
                      Enable click track during practice
                    </p>
                  </div>
                  <Switch
                    checked={preferences.metronomeEnabled}
                    onCheckedChange={(checked) => updatePreference('metronomeEnabled', checked)}
                  />
                </div>

                {preferences.metronomeEnabled && (
                  <div className="space-y-2">
                    <div className="flex items-center justify-between">
                      <Label className="text-sm font-medium" id="metronome-volume-label">Metronome Volume</Label>
                      <span className="text-sm font-medium text-primary">
                        {Math.round(preferences.metronomeVolume * 100)}%
                      </span>
                    </div>
                    <div className="flex items-center gap-3">
                      <span className="text-xs text-muted-foreground w-6">0%</span>
                      <Slider
                        min={0}
                        max={1}
                        step={0.05}
                        value={[preferences.metronomeVolume]}
                        onValueChange={(value) => updatePreference('metronomeVolume', value[0])}
                        aria-labelledby="metronome-volume-label"
                        aria-valuemin={0}
                        aria-valuemax={100}
                        aria-valuenow={Math.round(preferences.metronomeVolume * 100)}
                        aria-valuetext={`${Math.round(preferences.metronomeVolume * 100)} percent`}
                        className="flex-1"
                      />
                      <span className="text-xs text-muted-foreground w-10">100%</span>
                    </div>
                  </div>
                )}
              </div>
            )}

            {activeTab === 'voice' && (
              <div role="tabpanel" id="voice-panel" aria-labelledby="voice-tab" className="space-y-6">
                <div className="p-4 bg-blue-500/10 border border-blue-500/20 rounded-lg">
                  <div className="flex items-center gap-2 text-sm font-medium text-blue-400 mb-2">
                    <Mic className="h-4 w-4" />
                    Voice Commands
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Control Orpheus hands-free while playing your instrument. Say "Orpheus play", "Orpheus stop", or "Orpheus save".
                  </p>
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <Label className="text-sm font-medium">Enable Voice Commands</Label>
                    <p className="text-xs text-muted-foreground">
                      Allow voice control of playback, saving, and navigation
                    </p>
                  </div>
                  <Switch
                    checked={preferences.voiceEnabled}
                    onCheckedChange={(checked) => updatePreference('voiceEnabled', checked)}
                  />
                </div>

                {preferences.voiceEnabled && (
                  <>
                    <div className="flex items-center justify-between">
                      <div>
                        <Label className="text-sm font-medium">Wake Word Required</Label>
                        <p className="text-xs text-muted-foreground">
                          Require "Orpheus" before commands (recommended)
                        </p>
                      </div>
                      <Switch
                        checked={preferences.voiceWakeWordEnabled}
                        onCheckedChange={(checked) => updatePreference('voiceWakeWordEnabled', checked)}
                      />
                    </div>

                    <div className="flex items-center justify-between">
                      <div>
                        <Label className="text-sm font-medium">Continuous Listening</Label>
                        <p className="text-xs text-muted-foreground">
                          Keep listening after each command (battery intensive)
                        </p>
                      </div>
                      <Switch
                        checked={preferences.voiceContinuousMode}
                        onCheckedChange={(checked) => updatePreference('voiceContinuousMode', checked)}
                      />
                    </div>

                    <div className="flex items-center justify-between">
                      <div>
                        <Label className="text-sm font-medium">Audio Feedback</Label>
                        <p className="text-xs text-muted-foreground">
                          Play audio tones when commands are recognized
                        </p>
                      </div>
                      <Switch
                        checked={preferences.voiceAudioFeedback}
                        onCheckedChange={(checked) => updatePreference('voiceAudioFeedback', checked)}
                      />
                    </div>

                    <div className="p-3 bg-muted rounded-lg space-y-2">
                      <Label className="text-xs font-medium text-muted-foreground">Available Commands</Label>
                      <div className="grid grid-cols-2 gap-2 text-xs">
                        <div><strong>Playback:</strong> play, pause, stop, faster, slower</div>
                        <div><strong>Recording:</strong> count me in, punch in, good take</div>
                        <div><strong>Navigation:</strong> verse, chorus, bridge, intro</div>
                        <div><strong>Mixing:</strong> louder, quieter, solo, mute, pan</div>
                        <div><strong>Effects:</strong> reverb, delay, brighter, warmer</div>
                        <div><strong>Macros:</strong> ready to jam, wrap it up</div>
                      </div>
                    </div>
                  </>
                )}
              </div>
            )}

            {activeTab === 'help' && (
              <div role="tabpanel" id="help-panel" aria-labelledby="help-tab" className="space-y-6">
                <div className="p-4 bg-muted rounded-lg space-y-3">
                  <Label className="text-sm font-medium">Keyboard Shortcuts</Label>
                  <p className="text-xs text-muted-foreground">
                    Press <kbd className="px-1.5 py-0.5 bg-background border rounded text-[10px]">?</kbd> anywhere to see all keyboard shortcuts
                  </p>
                </div>

                <div className="space-y-3">
                  <Label className="text-sm font-medium">Tutorial</Label>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      resetOnboarding();
                      onClose();
                      // Trigger onboarding show
                      window.location.reload();
                    }}
                  >
                    <RotateCcw className="h-4 w-4 mr-2" />
                    Replay Onboarding Tutorial
                  </Button>
                </div>

                <div className="space-y-3">
                  <Label className="text-sm font-medium">About</Label>
                  <div className="text-sm text-muted-foreground space-y-1">
                    <p>Orpheus Music Platform v1.2.0</p>
                    <p>Built with React, Tailwind CSS, Tone.js, and alphaTab</p>
                  </div>
                </div>

                <div className="space-y-3">
                  <Label className="text-sm font-medium">Feedback</Label>
                  <p className="text-xs text-muted-foreground">
                    Found a bug or have a suggestion? Let us know!
                  </p>
                  <Button variant="outline" size="sm" asChild>
                    <a href="https://github.com/orpheus/feedback" target="_blank" rel="noopener noreferrer">
                      Submit Feedback
                    </a>
                  </Button>
                </div>
              </div>
            )}
          </div>
        </div>

        <DialogFooter className="flex-row justify-between sm:justify-between border-t pt-4">
          <Button variant="ghost" onClick={handleReset}>
            <RotateCcw className="h-4 w-4 mr-2" />
            Reset to Defaults
          </Button>
          <div className="flex gap-2">
            <Button variant="outline" onClick={onClose}>
              Cancel
            </Button>
            <Button onClick={handleSave} disabled={!hasChanges}>
              <Save className="h-4 w-4 mr-2" />
              Save Changes
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

/**
 * Hook to access preferences
 */
export function usePreferences() {
  const [preferences, setPreferences] = useState<UserPreferences>(loadPreferences);

  const updatePreference = <K extends keyof UserPreferences>(
    key: K,
    value: UserPreferences[K]
  ) => {
    setPreferences((prev) => {
      const updated = { ...prev, [key]: value };
      savePreferences(updated);
      return updated;
    });
  };

  return { preferences, updatePreference };
}
