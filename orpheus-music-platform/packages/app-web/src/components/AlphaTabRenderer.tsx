/**
 * AlphaTab Renderer Component
 * Professional tablature rendering using alphaTab library
 */

import { useEffect, useRef, useState, useCallback } from 'react';
import { Button, Slider, Tooltip, TooltipTrigger, TooltipContent } from '@persona-framework/ui';
import { Play, Pause, Square, SkipBack, SkipForward, Volume2, VolumeX } from 'lucide-react';
import * as alphaTab from '@coderline/alphatab';

interface AlphaTabRendererProps {
  gpData?: ArrayBuffer;
  visibleTracks?: number[];
  showControls?: boolean;
  onPositionChange?: (position: number, duration: number) => void;
  onTrackClick?: (trackIndex: number) => void;
}

export function AlphaTabRenderer({
  gpData,
  visibleTracks,
  showControls = true,
  onPositionChange,
  onTrackClick,
}: AlphaTabRendererProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const apiRef = useRef<alphaTab.AlphaTabApi | null>(null);

  const [isLoading, setIsLoading] = useState(true);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [totalTime, setTotalTime] = useState(0);
  const [volume, setVolume] = useState(1);
  const [isMuted, setIsMuted] = useState(false);
  const [tracks, setTracks] = useState<alphaTab.model.Track[]>([]);
  const [activeTracks, setActiveTracks] = useState<number[]>([]);

  useEffect(() => {
    if (!containerRef.current) return;

    const settings = new alphaTab.Settings();
    settings.core.engine = 'html5';
    settings.core.logLevel = alphaTab.LogLevel.Warning;
    settings.display.staveProfile = alphaTab.StaveProfile.Tab;
    settings.display.layoutMode = alphaTab.LayoutMode.Page;
    settings.notation.notationMode = alphaTab.NotationMode.GuitarPro;
    settings.player.enablePlayer = true;
    settings.player.enableCursor = true;
    settings.player.enableUserInteraction = true;
    settings.player.soundFont = '/soundfont/sonivox.sf2';

    const api = new alphaTab.AlphaTabApi(containerRef.current, settings);

    api.renderStarted.on(() => setIsLoading(true));
    api.renderFinished.on(() => setIsLoading(false));

    api.scoreLoaded.on((score) => {
      if (score) {
        setTracks(score.tracks);
        setActiveTracks(score.tracks.map((_, i) => i));
      }
    });

    api.playerStateChanged.on((args) => {
      setIsPlaying(args.state === alphaTab.synth.PlayerState.Playing);
    });

    api.playerPositionChanged.on((args) => {
      setCurrentTime(args.currentTime);
      setTotalTime(args.endTime);
      onPositionChange?.(args.currentTime, args.endTime);
    });

    apiRef.current = api;

    return () => {
      api.destroy();
      apiRef.current = null;
    };
  }, [onPositionChange]);

  useEffect(() => {
    if (!apiRef.current || !gpData) return;
    const uint8Array = new Uint8Array(gpData);
    apiRef.current.load(uint8Array);
  }, [gpData]);

  useEffect(() => {
    if (!apiRef.current || !tracks.length) return;
    if (visibleTracks) {
      const tracksToRender = tracks.filter((_, i) => visibleTracks.includes(i));
      apiRef.current.renderTracks(tracksToRender);
    }
  }, [visibleTracks, tracks]);

  const handlePlayPause = useCallback(() => {
    apiRef.current?.playPause();
  }, []);

  const handleStop = useCallback(() => {
    apiRef.current?.stop();
  }, []);

  const handlePrevious = useCallback(() => {
    if (!apiRef.current) return;
    apiRef.current.tickPosition = Math.max(0, apiRef.current.tickPosition - 960);
  }, []);

  const handleNext = useCallback(() => {
    if (!apiRef.current) return;
    apiRef.current.tickPosition = apiRef.current.tickPosition + 960;
  }, []);

  const handleVolumeChange = useCallback((value: number) => {
    if (!apiRef.current) return;
    setVolume(value);
    apiRef.current.masterVolume = value;
  }, []);

  const handleMuteToggle = useCallback(() => {
    if (!apiRef.current) return;
    const newMuted = !isMuted;
    setIsMuted(newMuted);
    apiRef.current.masterVolume = newMuted ? 0 : volume;
  }, [isMuted, volume]);

  const handleProgressClick = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    if (!apiRef.current || totalTime === 0) return;
    const rect = e.currentTarget.getBoundingClientRect();
    const percent = (e.clientX - rect.left) / rect.width;
    apiRef.current.timePosition = percent * totalTime;
  }, [totalTime]);

  const handleTrackToggle = useCallback((trackIndex: number) => {
    if (!apiRef.current) return;
    setActiveTracks(prev => {
      const newActive = prev.includes(trackIndex)
        ? prev.filter(i => i !== trackIndex)
        : [...prev, trackIndex];
      const tracksToRender = tracks.filter((_, i) => newActive.includes(i));
      if (tracksToRender.length > 0) {
        apiRef.current?.renderTracks(tracksToRender);
      }
      return newActive;
    });
    onTrackClick?.(trackIndex);
  }, [tracks, onTrackClick]);

  const formatTime = (ms: number) => {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
  };

  const progressPercent = totalTime > 0 ? (currentTime / totalTime) * 100 : 0;

  return (
    <div className="flex flex-col h-full bg-background rounded-lg overflow-hidden">
      {showControls && (
        <>
          <div className="flex items-center gap-2 px-4 py-3 bg-secondary border-b border-border">
            <div className="flex items-center gap-1">
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button variant="ghost" size="icon" onClick={handlePrevious}>
                    <SkipBack className="h-4 w-4" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Previous</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button onClick={handlePlayPause}>
                    {isPlaying ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />}
                  </Button>
                </TooltipTrigger>
                <TooltipContent>{isPlaying ? 'Pause' : 'Play'}</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button variant="ghost" size="icon" onClick={handleStop}>
                    <Square className="h-4 w-4" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Stop</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button variant="ghost" size="icon" onClick={handleNext}>
                    <SkipForward className="h-4 w-4" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Next</TooltipContent>
              </Tooltip>
            </div>

            <div className="font-mono text-sm text-muted-foreground min-w-[100px] text-center">
              {formatTime(currentTime)} / {formatTime(totalTime)}
            </div>

            {tracks.length > 1 && (
              <div className="flex gap-1 flex-wrap ml-4">
                {tracks.map((track, i) => (
                  <Button
                    key={i}
                    size="sm"
                    variant={activeTracks.includes(i) ? 'default' : 'outline'}
                    onClick={() => handleTrackToggle(i)}
                  >
                    {track.name || `Track ${i + 1}`}
                  </Button>
                ))}
              </div>
            )}

            <div className="flex items-center gap-2 ml-auto w-[150px]">
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button variant="ghost" size="icon" onClick={handleMuteToggle}>
                    {isMuted ? <VolumeX className="h-4 w-4" /> : <Volume2 className="h-4 w-4" />}
                  </Button>
                </TooltipTrigger>
                <TooltipContent>{isMuted ? 'Unmute' : 'Mute'}</TooltipContent>
              </Tooltip>
              <Slider
                min={0}
                max={1}
                step={0.05}
                value={[isMuted ? 0 : volume]}
                onValueChange={(value) => handleVolumeChange(value[0])}
                className="flex-1"
              />
            </div>
          </div>

          <div className="px-4 bg-secondary">
            <div
              className="w-full h-1 bg-muted rounded cursor-pointer relative"
              onClick={handleProgressClick}
            >
              <div
                className="absolute h-full bg-primary rounded transition-[width] duration-100"
                style={{ width: `${progressPercent}%` }}
              />
            </div>
          </div>
        </>
      )}

      <div className="flex-1 overflow-auto p-4">
        {isLoading && !gpData && (
          <div className="flex items-center justify-center h-[300px] text-muted-foreground">
            Drop a Guitar Pro file or import one to view tablature
          </div>
        )}
        <div ref={containerRef} className="w-full min-h-[400px]" />
      </div>
    </div>
  );
}
