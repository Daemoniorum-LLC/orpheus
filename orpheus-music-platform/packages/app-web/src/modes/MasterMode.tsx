/**
 * Master Mode - AI-powered mastering (Nexus DAW)
 */

import {
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@persona-framework/ui';
import { Download, Bot, FolderOpen } from 'lucide-react';
import { useState, useEffect } from 'react';
import { useProject, useAppStore } from '../store/app-store';
import { importFile } from '../services/file-import';
import { MasteringChain, type MasteringChainSettings } from '../components/MasteringChain';
import { LUFSMeter } from '../components/LUFSMeter';
import { getAudioProcessingManager } from '../services/audio-processing';

export function MasterMode() {
  const project = useProject();
  const { setAIAssistantOpen, setProject, setMode } = useAppStore();
  const [selectedPlatform, setSelectedPlatform] = useState('spotify');
  const [masteringSettings, setMasteringSettings] = useState<MasteringChainSettings | null>(null);

  // Wire up mastering chain to audio processing
  useEffect(() => {
    if (masteringSettings) {
      const manager = getAudioProcessingManager();
      const master = manager.getMaster();
      master.updateChain(masteringSettings);
    }
  }, [masteringSettings]);

  const platforms = [
    { id: 'spotify', name: 'Spotify', target: -14, color: '#1DB954' },
    { id: 'apple', name: 'Apple Music', target: -16, color: '#FA243C' },
    { id: 'youtube', name: 'YouTube', target: -14, color: '#FF0000' },
    { id: 'tidal', name: 'Tidal', target: -14, color: '#000000' },
    { id: 'soundcloud', name: 'SoundCloud', target: -8, color: '#FF5500' },
  ];

  const exportFormats = [
    { name: 'WAV', format: '44.1kHz/24-bit', size: '~50MB' },
    { name: 'WAV', format: '48kHz/24-bit', size: '~55MB' },
    { name: 'FLAC', format: 'Lossless', size: '~35MB' },
    { name: 'MP3', format: '320kbps', size: '~12MB' },
    { name: 'AAC', format: '256kbps', size: '~10MB' },
  ];

  const handleImportFile = async () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.gp,.gpx,.gp5,.gp4,.gp3,.maestro';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        const result = await importFile(file);
        if (result.success && result.project) {
          setProject(result.project);
        }
      }
    };
    input.click();
  };

  if (!project) {
    return (
      <div className="h-full flex flex-col">
        <div className="p-4 border-b border-border flex justify-between items-center">
          <div className="text-xl font-semibold">✨ Master - AI Mastering</div>
        </div>
        <div className="flex-1 flex items-center justify-center flex-col gap-4">
          <h3 className="text-lg font-semibold">No Project Loaded</h3>
          <p className="text-muted-foreground mb-6">
            To use Master mode, you need to load a project first.
          </p>
          <div className="flex gap-3 justify-center">
            <Button onClick={handleImportFile}>
              <FolderOpen className="h-4 w-4 mr-2" />
              Import Guitar Pro File
            </Button>
            <Button variant="outline" onClick={() => setMode('compose')}>
              Go to Compose Mode
            </Button>
          </div>
          <p className="text-xs text-muted-foreground mt-4">
            Or press <strong>Ctrl+1</strong> to switch to Compose mode
          </p>
        </div>
      </div>
    );
  }

  const selectedTarget = platforms.find((p) => p.id === selectedPlatform);

  return (
    <div className="h-full flex flex-col">
      <div className="p-4 border-b border-border flex justify-between items-center">
        <div className="text-xl font-semibold">✨ Master - AI Mastering</div>
        <Button onClick={() => setAIAssistantOpen(true)}>
          <Bot className="h-4 w-4 mr-2" />
          AI Auto-Master
        </Button>
      </div>

      <div className="flex-1 flex p-6 gap-6 overflow-auto">
        {/* Left Panel - Meters */}
        <div className="flex-1 flex flex-col gap-5">
          {/* Professional LUFS Metering */}
          <LUFSMeter
            target={selectedTarget?.target || -14}
            showTargets={true}
          />

          {/* Mastering Chain */}
          <Card>
            <CardContent className="p-5">
              <MasteringChain onSettingsChange={setMasteringSettings} />
            </CardContent>
          </Card>
        </div>

        {/* Right Panel - Platform Targets & Export */}
        <div className="w-[400px] flex flex-col gap-5">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Platform Targets</CardTitle>
            </CardHeader>
            <CardContent>
              <Select value={selectedPlatform} onValueChange={setSelectedPlatform}>
                <SelectTrigger className="mb-4">
                  <SelectValue placeholder="Select platform" />
                </SelectTrigger>
                <SelectContent>
                  {platforms.map((p) => (
                    <SelectItem key={p.id} value={p.id}>
                      {p.name} ({p.target} LUFS)
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>

              <div className="flex flex-col gap-3">
                {platforms.map((platform) => (
                  <div
                    key={platform.id}
                    className="flex justify-between items-center p-3 bg-secondary rounded-md"
                  >
                    <span className="font-semibold">{platform.name}</span>
                    <span className="font-mono" style={{ color: platform.color }}>
                      {platform.target} LUFS
                    </span>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-base">Export</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex flex-col gap-2">
                {exportFormats.map((fmt, idx) => (
                  <Button
                    key={idx}
                    variant="ghost"
                    className="justify-start w-full"
                  >
                    <Download className="h-4 w-4 mr-2" />
                    <div className="flex-1 flex justify-between">
                      <span>
                        {fmt.name} - {fmt.format}
                      </span>
                      <span className="text-muted-foreground text-[11px]">
                        {fmt.size}
                      </span>
                    </div>
                  </Button>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
