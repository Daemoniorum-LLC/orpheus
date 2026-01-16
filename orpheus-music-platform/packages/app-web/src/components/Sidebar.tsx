/**
 * Sidebar - Track list and project navigation
 */

import { Button } from '@persona-framework/ui';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { useAppStore, useProject } from '../store/app-store';
import { useIsDesktop } from '../hooks/useMediaQuery';
import { useEffect, useRef } from 'react';

export function Sidebar() {
  const {
    sidebarOpen,
    setSidebarOpen,
    selectedTrackId,
    setSelectedTrackId,
  } = useAppStore();
  const project = useProject();
  const isDesktop = useIsDesktop();

  // Track if we've done the initial mobile collapse (only do it once)
  const hasInitializedRef = useRef(false);

  // Auto-collapse sidebar on initial load for smaller screens (once only)
  useEffect(() => {
    if (!hasInitializedRef.current && !isDesktop && sidebarOpen) {
      setSidebarOpen(false);
      hasInitializedRef.current = true;
    } else if (isDesktop) {
      hasInitializedRef.current = true;
    }
  }, [isDesktop, sidebarOpen, setSidebarOpen]);

  const tracks = project?.project.composition.tracks || [];

  return (
    <>
      <div
        className={`relative flex flex-col bg-secondary border-r border-border transition-all duration-200 ${
          sidebarOpen ? 'w-[280px]' : 'w-0 overflow-hidden'
        }`}
      >
        <div className="p-4 border-b border-border text-base font-semibold">
          Tracks
        </div>

        <div className="flex-1 overflow-auto p-2">
          {tracks.length === 0 ? (
            <div className="p-4 text-muted-foreground text-center text-sm">
              No tracks yet.
              <br />
              Import a Guitar Pro file to get started!
            </div>
          ) : (
            tracks.map((track: any) => (
              <div
                key={track.id}
                role="button"
                tabIndex={0}
                className={`p-3 my-1 rounded cursor-pointer transition-colors ${
                  selectedTrackId === track.id
                    ? 'bg-primary/20'
                    : 'bg-background hover:bg-muted'
                } focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2`}
                onClick={() => setSelectedTrackId(track.id)}
                onKeyDown={(e) => e.key === 'Enter' && setSelectedTrackId(track.id)}
                aria-selected={selectedTrackId === track.id}
              >
                <div className="text-sm font-semibold mb-1 truncate" title={track.name}>{track.name}</div>
                <div className="text-xs text-muted-foreground truncate">
                  {track.instrument?.type || 'Unknown'} •{' '}
                  {track.instrument?.tuning?.join('-') || 'Standard'}
                </div>
              </div>
            ))
          )}
        </div>

        <Button
          variant="ghost"
          size="icon"
          className="absolute -right-4 top-1/2 -translate-y-1/2 z-10 rounded-full h-8 w-8 bg-background border border-border shadow-sm"
          onClick={() => setSidebarOpen(!sidebarOpen)}
          aria-label={sidebarOpen ? 'Collapse sidebar' : 'Expand sidebar'}
          aria-expanded={sidebarOpen}
        >
          {sidebarOpen ? (
            <ChevronLeft className="h-4 w-4" aria-hidden="true" />
          ) : (
            <ChevronRight className="h-4 w-4" aria-hidden="true" />
          )}
        </Button>
      </div>
    </>
  );
}
