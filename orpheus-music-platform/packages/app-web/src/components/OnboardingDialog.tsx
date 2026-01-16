/**
 * Onboarding Dialog - Interactive first-run tutorial for new users
 * Features: Interactive demos, keyboard shortcut visualization, guided actions
 */

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
  Button,
} from '@persona-framework/ui';
import {
  Music,
  Mic,
  SlidersHorizontal,
  Sparkles,
  ArrowRight,
  Check,
  Upload,
  Keyboard,
  Play,
  Target,
  Share2,
  Wand2,
  ChevronRight,
  RotateCcw,
} from 'lucide-react';
import { useState, useRef, useCallback } from 'react';
import { cn } from '../lib/utils';
import type { AppMode } from '@orpheus/shared-types';
import { safeStorage } from '../utils/storage';

interface OnboardingDialogProps {
  open: boolean;
  onClose: () => void;
  onModeChange?: (mode: AppMode) => void;
  onImportFile?: () => void;
}

// Keyboard shortcut display component
function KeyboardShortcut({ keys, label }: { keys: string[]; label: string }) {
  return (
    <div className="flex items-center gap-2 text-sm">
      <div className="flex gap-1">
        {keys.map((key, i) => (
          <kbd
            key={i}
            className="px-2 py-1 bg-background border border-border rounded text-xs font-mono shadow-sm"
          >
            {key}
          </kbd>
        ))}
      </div>
      <span className="text-muted-foreground">{label}</span>
    </div>
  );
}

// Interactive mode card with hover effects
function ModeCard({
  icon: Icon,
  name,
  shortcut,
  description,
  isActive,
  onClick,
}: {
  icon: React.ElementType;
  name: string;
  shortcut: string;
  description: string;
  isActive: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        'flex items-start gap-3 p-3 rounded-lg border text-left transition-all',
        'hover:border-primary hover:bg-primary/5',
        isActive ? 'border-primary bg-primary/10' : 'border-border bg-background'
      )}
    >
      <div
        className={cn(
          'p-2 rounded-lg',
          isActive ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground'
        )}
      >
        <Icon className="h-4 w-4" />
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-medium text-sm">{name}</span>
          <kbd className="px-1.5 py-0.5 bg-muted rounded text-[10px] font-mono">
            {shortcut}
          </kbd>
        </div>
        <p className="text-xs text-muted-foreground mt-0.5 line-clamp-2">{description}</p>
      </div>
      <ChevronRight
        className={cn(
          'h-4 w-4 transition-transform',
          isActive ? 'text-primary translate-x-1' : 'text-muted-foreground'
        )}
      />
    </button>
  );
}

// Step content definitions
const MODES = [
  {
    id: 'compose' as AppMode,
    icon: Music,
    name: 'Compose',
    shortcut: 'Ctrl+1',
    description: 'Tab editor with Guitar Pro import & alphaTab rendering',
  },
  {
    id: 'record' as AppMode,
    icon: Mic,
    name: 'Record',
    shortcut: 'Ctrl+2',
    description: 'Multi-track audio recording with waveform display',
  },
  {
    id: 'mix' as AppMode,
    icon: SlidersHorizontal,
    name: 'Mix',
    shortcut: 'Ctrl+3',
    description: 'Professional mixing console with effects chain',
  },
  {
    id: 'master' as AppMode,
    icon: Target,
    name: 'Master',
    shortcut: 'Ctrl+4',
    description: 'AI-powered mastering with LUFS metering',
  },
  {
    id: 'practice' as AppMode,
    icon: Play,
    name: 'Practice',
    shortcut: 'Ctrl+5',
    description: 'Speed trainer with loop sections',
  },
  {
    id: 'distribute' as AppMode,
    icon: Share2,
    name: 'Distribute',
    shortcut: 'Ctrl+6',
    description: 'Publish to streaming platforms',
  },
];

export function OnboardingDialog({
  open,
  onClose,
  onModeChange,
  onImportFile,
}: OnboardingDialogProps) {
  const [currentStep, setCurrentStep] = useState(0);
  const [selectedMode, setSelectedMode] = useState<AppMode>('compose');
  const [hasTriedShortcut, setHasTriedShortcut] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const totalSteps = 4;
  const isLastStep = currentStep === totalSteps - 1;

  const handleNext = () => {
    if (currentStep < totalSteps - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      completeOnboarding();
    }
  };

  const handleBack = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleSkip = () => {
    completeOnboarding();
  };

  const completeOnboarding = () => {
    safeStorage.setItem('orpheus-onboarding-complete', 'true');
    onClose();
  };

  const handleModeClick = (mode: AppMode) => {
    setSelectedMode(mode);
    setHasTriedShortcut(true);
  };

  const handleTryMode = () => {
    if (onModeChange) {
      onModeChange(selectedMode);
    }
    completeOnboarding();
  };

  const handleImportClick = () => {
    if (onImportFile) {
      onImportFile();
      completeOnboarding();
    } else {
      fileInputRef.current?.click();
    }
  };

  // Keyboard shortcut listener for step 2
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (currentStep === 1 && e.ctrlKey) {
        const modeIndex = parseInt(e.key) - 1;
        if (modeIndex >= 0 && modeIndex < MODES.length) {
          e.preventDefault();
          setSelectedMode(MODES[modeIndex].id);
          setHasTriedShortcut(true);
        }
      }
    },
    [currentStep]
  );

  const renderStepContent = () => {
    switch (currentStep) {
      case 0:
        return (
          <div className="flex flex-col items-center text-center gap-6 py-4">
            {/* Animated logo */}
            <div className="relative">
              <div className="absolute inset-0 bg-gradient-to-br from-primary/30 to-purple-500/30 rounded-full blur-xl animate-pulse" />
              <div className="relative p-6 bg-gradient-to-br from-primary to-purple-500 rounded-full">
                <Music className="h-12 w-12 text-white" />
              </div>
            </div>

            <div>
              <h2 className="text-3xl font-bold mb-2">Welcome to Orpheus</h2>
              <p className="text-muted-foreground max-w-[400px] leading-relaxed">
                Your complete music production suite — from first chord to final master.
              </p>
            </div>

            {/* Quick start action */}
            <div className="w-full max-w-[350px] p-4 border border-dashed border-primary/50 rounded-lg bg-primary/5">
              <p className="text-sm font-medium mb-3">
                Have a Guitar Pro file? Start right away!
              </p>
              <Button onClick={handleImportClick} className="w-full">
                <Upload className="h-4 w-4 mr-2" />
                Import Guitar Pro File
              </Button>
              <p className="text-xs text-muted-foreground mt-2">
                Supports .gp3, .gp4, .gp5, .gpx, .gp formats
              </p>
              <input
                ref={fileInputRef}
                type="file"
                accept=".gp3,.gp4,.gp5,.gpx,.gp"
                className="hidden"
                onChange={() => completeOnboarding()}
              />
            </div>

            <p className="text-sm text-muted-foreground">
              Or continue the tour to learn about all features →
            </p>
          </div>
        );

      case 1:
        return (
          <div
            className="flex flex-col gap-4 py-2"
            onKeyDown={handleKeyDown}
            tabIndex={0}
          >
            <div className="text-center mb-2">
              <h2 className="text-2xl font-semibold mb-1">6 Powerful Modes</h2>
              <p className="text-sm text-muted-foreground">
                Click a mode or try the keyboard shortcut
              </p>
            </div>

            {/* Mode grid */}
            <div className="grid grid-cols-2 gap-2">
              {MODES.map((mode) => (
                <ModeCard
                  key={mode.id}
                  icon={mode.icon}
                  name={mode.name}
                  shortcut={mode.shortcut}
                  description={mode.description}
                  isActive={selectedMode === mode.id}
                  onClick={() => handleModeClick(mode.id)}
                />
              ))}
            </div>

            {/* Try it prompt */}
            {hasTriedShortcut && (
              <div className="flex items-center justify-center gap-3 p-3 bg-green-500/10 border border-green-500/30 rounded-lg animate-in fade-in slide-in-from-bottom-2">
                <Check className="h-5 w-5 text-green-500" />
                <span className="text-sm font-medium text-green-700 dark:text-green-400">
                  Great! You selected {MODES.find((m) => m.id === selectedMode)?.name} mode
                </span>
              </div>
            )}

            <div className="text-center">
              <Button variant="outline" size="sm" onClick={handleTryMode}>
                Jump to {MODES.find((m) => m.id === selectedMode)?.name} Mode
                <ArrowRight className="h-4 w-4 ml-2" />
              </Button>
            </div>
          </div>
        );

      case 2:
        return (
          <div className="flex flex-col items-center text-center gap-6 py-4">
            <div className="p-4 bg-gradient-to-br from-purple-500/20 to-pink-500/20 rounded-full">
              <Wand2 className="h-12 w-12 text-purple-500" />
            </div>

            <div>
              <h2 className="text-2xl font-semibold mb-2">AI Assistant</h2>
              <p className="text-muted-foreground max-w-[400px] leading-relaxed">
                Get intelligent help tailored to your current workflow.
              </p>
            </div>

            {/* AI features showcase */}
            <div className="w-full space-y-3">
              <div className="flex items-center gap-3 p-3 bg-muted rounded-lg text-left">
                <Sparkles className="h-5 w-5 text-primary flex-shrink-0" />
                <div>
                  <div className="text-sm font-medium">Context-Aware Suggestions</div>
                  <div className="text-xs text-muted-foreground">
                    AI adapts to your current mode and task
                  </div>
                </div>
              </div>
              <div className="flex items-center gap-3 p-3 bg-muted rounded-lg text-left">
                <Keyboard className="h-5 w-5 text-primary flex-shrink-0" />
                <div>
                  <div className="text-sm font-medium">Quick Access</div>
                  <div className="text-xs text-muted-foreground">
                    Press <kbd className="px-1 py-0.5 bg-background border rounded text-[10px]">Ctrl</kbd> + <kbd className="px-1 py-0.5 bg-background border rounded text-[10px]">K</kbd> anywhere to open
                  </div>
                </div>
              </div>
              <div className="flex items-center gap-3 p-3 bg-muted rounded-lg text-left">
                <Music className="h-5 w-5 text-primary flex-shrink-0" />
                <div>
                  <div className="text-sm font-medium">Music Intelligence</div>
                  <div className="text-xs text-muted-foreground">
                    Chord suggestions, mixing tips, mastering advice
                  </div>
                </div>
              </div>
            </div>
          </div>
        );

      case 3:
        return (
          <div className="flex flex-col items-center text-center gap-6 py-4">
            <div className="p-4 bg-gradient-to-br from-green-500/20 to-emerald-500/20 rounded-full">
              <Check className="h-12 w-12 text-green-500" />
            </div>

            <div>
              <h2 className="text-2xl font-semibold mb-2">You're All Set!</h2>
              <p className="text-muted-foreground max-w-[400px] leading-relaxed">
                Here are the essential shortcuts to remember:
              </p>
            </div>

            {/* Essential shortcuts */}
            <div className="w-full space-y-2 p-4 bg-muted rounded-lg">
              <KeyboardShortcut keys={['Ctrl', '1-6']} label="Switch between modes" />
              <KeyboardShortcut keys={['Ctrl', 'K']} label="Open AI Assistant" />
              <KeyboardShortcut keys={['Ctrl', 'S']} label="Save project" />
              <KeyboardShortcut keys={['Ctrl', 'O']} label="Open project" />
              <KeyboardShortcut keys={['Space']} label="Play/Pause" />
            </div>

            {/* Replay option */}
            <p className="text-xs text-muted-foreground flex items-center gap-1">
              <RotateCcw className="h-3 w-3" />
              You can replay this tutorial from Settings → Help
            </p>
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <Dialog open={open} onOpenChange={(isOpen) => !isOpen && handleSkip()}>
      <DialogContent className="max-w-[550px] max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          {/* Step indicator */}
          <div className="flex justify-center gap-2 mb-2">
            {Array.from({ length: totalSteps }).map((_, index) => (
              <button
                key={index}
                onClick={() => setCurrentStep(index)}
                className={cn(
                  'w-2.5 h-2.5 rounded-full transition-all',
                  index === currentStep
                    ? 'bg-primary w-6'
                    : index < currentStep
                      ? 'bg-primary/60'
                      : 'bg-muted hover:bg-muted-foreground/30'
                )}
                aria-label={`Go to step ${index + 1}`}
              />
            ))}
          </div>
          <DialogTitle className="sr-only">Onboarding Step {currentStep + 1}</DialogTitle>
          <DialogDescription className="sr-only">
            Learn how to use Orpheus music production platform
          </DialogDescription>
        </DialogHeader>

        {renderStepContent()}

        <DialogFooter className="flex-row justify-between sm:justify-between">
          <div>
            {currentStep > 0 ? (
              <Button variant="ghost" onClick={handleBack}>
                Back
              </Button>
            ) : (
              <Button variant="ghost" onClick={handleSkip}>
                Skip Tutorial
              </Button>
            )}
          </div>
          <Button onClick={handleNext}>
            {isLastStep ? (
              "Let's Go!"
            ) : (
              <>
                Next
                <ArrowRight className="h-4 w-4 ml-2" />
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

/**
 * Hook to manage onboarding state
 */
export function useOnboarding() {
  const isComplete = safeStorage.getItem('orpheus-onboarding-complete') === 'true';

  const resetOnboarding = () => {
    safeStorage.removeItem('orpheus-onboarding-complete');
  };

  return {
    shouldShowOnboarding: !isComplete,
    resetOnboarding,
  };
}
