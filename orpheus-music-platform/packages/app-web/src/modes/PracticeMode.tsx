/**
 * Practice Mode - Speed trainer and learning tools
 */

import { Button } from '@persona-framework/ui';
import { Bot } from 'lucide-react';
import { useAppStore } from '../store/app-store';
import { SpeedTrainer } from '../components/SpeedTrainer';

export function PracticeMode() {
  const { setAIAssistantOpen } = useAppStore();

  return (
    <div className="h-full flex flex-col">
      <div className="p-4 border-b border-border flex justify-between items-center">
        <div className="text-xl font-semibold">🎸 Practice - Learning Tools</div>
        <Button variant="ghost" onClick={() => setAIAssistantOpen(true)}>
          <Bot className="h-4 w-4 mr-2" />
          AI Practice Coach
        </Button>
      </div>
      <div className="flex-1 flex flex-col items-center justify-center p-6 gap-6">
        <div className="text-center max-w-[600px] text-muted-foreground">
          <h2 className="text-xl font-semibold text-foreground">Speed Trainer & Progressive Practice</h2>
          <p className="mt-3 leading-relaxed">
            Build speed gradually with automatic tempo increments.
            <br />
            Practice slowly and perfectly, then watch your speed increase!
          </p>
        </div>

        <div className="w-full max-w-[800px]">
          <SpeedTrainer
            initialBPM={60}
            targetBPM={120}
            incrementStep={5}
            repsBeforeIncrement={3}
          />
        </div>
      </div>
    </div>
  );
}
