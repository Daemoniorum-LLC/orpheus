/**
 * Input Level Meter - Real-time audio input level indicator
 */

interface InputLevelMeterProps {
  level: number; // 0-1
  showLabel?: boolean;
}

export function InputLevelMeter({ level, showLabel = true }: InputLevelMeterProps) {
  // Determine color based on level
  const getColor = (level: number): string => {
    if (level < 0.3) return 'bg-green-500'; // Low - green
    if (level < 0.7) return 'bg-yellow-500'; // Medium - yellow
    if (level < 0.9) return 'bg-orange-500'; // High - orange
    return 'bg-red-500'; // Clipping - red
  };

  const getStatus = (level: number): string => {
    if (level < 0.3) return 'Low';
    if (level < 0.7) return 'Good';
    if (level < 0.9) return 'High';
    return 'CLIPPING!';
  };

  const percentage = Math.round(level * 100);

  return (
    <div className="flex flex-col gap-2">
      {showLabel && (
        <div className="text-xs font-semibold text-muted-foreground">
          Input Level: {getStatus(level)}
        </div>
      )}
      <div className="w-full h-6 bg-muted rounded border border-border overflow-hidden relative">
        <div
          className={`h-full transition-[width] duration-50 ${getColor(level)}`}
          style={{ width: `${percentage}%` }}
        />
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-[11px] font-semibold text-foreground drop-shadow-[0_0_2px_rgba(0,0,0,0.5)] pointer-events-none">
          {percentage}%
        </div>
      </div>
    </div>
  );
}
