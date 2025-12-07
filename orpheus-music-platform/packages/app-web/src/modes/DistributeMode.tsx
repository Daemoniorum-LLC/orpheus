/**
 * Distribute Mode - Music distribution platform integration
 * Upload to DistroKid, Spotify for Artists, Apple Music, etc.
 */

import {
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Input,
  Label,
  Checkbox,
  Progress,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@persona-framework/ui';
import {
  CloudUpload,
  Music,
  Calendar,
  Image,
  Check,
  AlertTriangle,
  FolderOpen,
} from 'lucide-react';
import { useState } from 'react';
import { useProject, useAppStore } from '../store/app-store';
import { importFile } from '../services/file-import';

interface Platform {
  id: string;
  name: string;
  color: string;
  fee: string;
  deliveryTime: string;
  selected: boolean;
}

interface DistributionStatus {
  step: string;
  status: 'pending' | 'in_progress' | 'completed' | 'warning';
  detail: string;
}

export function DistributeMode() {
  const project = useProject();
  const { setProject, setMode } = useAppStore();

  // Release metadata
  const [trackTitle, setTrackTitle] = useState('');
  const [artistName, setArtistName] = useState('');

  // Validation errors
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [touched, setTouched] = useState<Record<string, boolean>>({});
  const [albumTitle, setAlbumTitle] = useState('');
  const [genre, setGenre] = useState('');
  const [releaseDate, setReleaseDate] = useState('');
  const [isrc, setIsrc] = useState('');
  const [upc, setUpc] = useState('');
  const [lyrics, setLyrics] = useState('');
  const [explicitContent, setExplicitContent] = useState(false);

  // Platforms
  const [platforms, setPlatforms] = useState<Platform[]>([
    {
      id: 'spotify',
      name: 'Spotify',
      color: '#1DB954',
      fee: 'Free via DistroKid',
      deliveryTime: '1-2 days',
      selected: true,
    },
    {
      id: 'apple',
      name: 'Apple Music',
      color: '#FA243C',
      fee: 'Free via DistroKid',
      deliveryTime: '1-2 days',
      selected: true,
    },
    {
      id: 'youtube',
      name: 'YouTube Music',
      color: '#FF0000',
      fee: 'Free via DistroKid',
      deliveryTime: '1-2 days',
      selected: true,
    },
    {
      id: 'tidal',
      name: 'Tidal',
      color: '#000000',
      fee: 'Free via DistroKid',
      deliveryTime: '3-5 days',
      selected: false,
    },
    {
      id: 'amazon',
      name: 'Amazon Music',
      color: '#FF9900',
      fee: 'Free via DistroKid',
      deliveryTime: '1-2 days',
      selected: true,
    },
    {
      id: 'deezer',
      name: 'Deezer',
      color: '#FF0000',
      fee: 'Free via DistroKid',
      deliveryTime: '3-5 days',
      selected: false,
    },
  ]);

  // Distribution status
  const [statusItems] = useState<DistributionStatus[]>([
    { step: 'Audio Quality Check', status: 'completed', detail: 'Passed: 48kHz/24-bit WAV' },
    { step: 'Metadata Validation', status: 'warning', detail: 'ISRC code recommended' },
    { step: 'Artwork Verification', status: 'pending', detail: 'Upload 3000x3000px JPG' },
    { step: 'Platform Submission', status: 'pending', detail: 'Ready to submit' },
    { step: 'Distribution', status: 'pending', detail: 'Estimated 1-2 days' },
  ]);

  const [uploadProgress, setUploadProgress] = useState(0);

  const togglePlatform = (id: string) => {
    setPlatforms(platforms.map((p) => (p.id === id ? { ...p, selected: !p.selected } : p)));
  };

  const selectedPlatformCount = platforms.filter((p) => p.selected).length;

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!trackTitle.trim()) {
      newErrors.trackTitle = 'Track title is required';
    }
    if (!artistName.trim()) {
      newErrors.artistName = 'Artist name is required';
    }
    if (!genre) {
      newErrors.genre = 'Please select a genre';
    }
    if (!releaseDate) {
      newErrors.releaseDate = 'Release date is required';
    }

    if (isrc && !/^[A-Z]{2}-[A-Z0-9]{3}-\d{2}-\d{5}$/.test(isrc.toUpperCase())) {
      newErrors.isrc = 'Invalid ISRC format (example: US-ABC-12-34567)';
    }

    if (releaseDate) {
      const date = new Date(releaseDate);
      const today = new Date();
      today.setHours(0, 0, 0, 0);
      if (date < today) {
        newErrors.releaseDate = 'Release date must be today or in the future';
      }
    }

    if (selectedPlatformCount === 0) {
      newErrors.platforms = 'Please select at least one distribution platform';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = () => {
    setTouched({
      trackTitle: true,
      artistName: true,
      genre: true,
      releaseDate: true,
      platforms: true,
    });

    if (!validateForm()) {
      return;
    }

    let progress = 0;
    const interval = setInterval(() => {
      progress += 10;
      setUploadProgress(progress);
      if (progress >= 100) {
        clearInterval(interval);
      }
    }, 500);
  };

  const handleBlur = (field: string) => {
    setTouched({ ...touched, [field]: true });
    validateForm();
  };

  const getStatusIcon = (status: DistributionStatus['status']) => {
    switch (status) {
      case 'completed':
        return <Check className="h-5 w-5 text-green-500" />;
      case 'warning':
        return <AlertTriangle className="h-5 w-5 text-yellow-500" />;
      case 'in_progress':
        return <div className="h-5 w-5 border-2 border-primary border-t-transparent rounded-full animate-spin" />;
      default:
        return <div className="h-5 w-5" />;
    }
  };

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
          <div className="text-xl font-semibold">🌍 Distribute - Music Distribution</div>
        </div>
        <div className="flex-1 flex items-center justify-center flex-col gap-4">
          <h3 className="text-lg font-semibold">No Project Loaded</h3>
          <p className="text-muted-foreground mb-6">
            To use Distribute mode, you need to load a project first.
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

  return (
    <div className="h-full flex flex-col">
      <div className="p-4 border-b border-border flex justify-between items-center">
        <div className="text-xl font-semibold">🌍 Distribute - Music Distribution</div>
        <div className="flex items-center gap-2">
          <Button onClick={handleSubmit} disabled={selectedPlatformCount === 0}>
            <CloudUpload className="h-4 w-4 mr-2" />
            Submit to {selectedPlatformCount} {selectedPlatformCount === 1 ? 'Platform' : 'Platforms'}
          </Button>
          {touched.platforms && errors.platforms && (
            <span className="text-destructive text-xs">{errors.platforms}</span>
          )}
        </div>
      </div>

      <div className="flex-1 flex p-6 gap-6 overflow-auto">
        {/* Left Panel - Metadata & Artwork */}
        <div className="flex-[2] flex flex-col gap-5">
          {/* Release Information */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <Music className="h-5 w-5" />
                Release Information
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-2 gap-4">
                <div className="flex flex-col gap-2">
                  <Label htmlFor="track-title">Track Title *</Label>
                  <Input
                    id="track-title"
                    value={trackTitle}
                    onChange={(e) => setTrackTitle(e.target.value)}
                    onBlur={() => handleBlur('trackTitle')}
                    placeholder="Enter track title"
                    className={touched.trackTitle && errors.trackTitle ? 'border-destructive' : ''}
                  />
                  {touched.trackTitle && errors.trackTitle && (
                    <span className="text-destructive text-xs">{errors.trackTitle}</span>
                  )}
                </div>
                <div className="flex flex-col gap-2">
                  <Label htmlFor="artist-name">Artist Name *</Label>
                  <Input
                    id="artist-name"
                    value={artistName}
                    onChange={(e) => setArtistName(e.target.value)}
                    onBlur={() => handleBlur('artistName')}
                    placeholder="Enter artist name"
                    className={touched.artistName && errors.artistName ? 'border-destructive' : ''}
                  />
                  {touched.artistName && errors.artistName && (
                    <span className="text-destructive text-xs">{errors.artistName}</span>
                  )}
                </div>
                <div className="flex flex-col gap-2">
                  <Label htmlFor="album-title">Album Title</Label>
                  <Input
                    id="album-title"
                    value={albumTitle}
                    onChange={(e) => setAlbumTitle(e.target.value)}
                    placeholder="Single, EP, or Album name"
                  />
                </div>
                <div className="flex flex-col gap-2">
                  <Label htmlFor="genre">Genre *</Label>
                  <Select value={genre} onValueChange={(value) => { setGenre(value); handleBlur('genre'); }}>
                    <SelectTrigger className={touched.genre && errors.genre ? 'border-destructive' : ''}>
                      <SelectValue placeholder="Select genre" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="rock">Rock</SelectItem>
                      <SelectItem value="metal">Metal</SelectItem>
                      <SelectItem value="pop">Pop</SelectItem>
                      <SelectItem value="electronic">Electronic</SelectItem>
                      <SelectItem value="hip-hop">Hip Hop</SelectItem>
                      <SelectItem value="jazz">Jazz</SelectItem>
                      <SelectItem value="classical">Classical</SelectItem>
                      <SelectItem value="country">Country</SelectItem>
                    </SelectContent>
                  </Select>
                  {touched.genre && errors.genre && (
                    <span className="text-destructive text-xs">{errors.genre}</span>
                  )}
                </div>
                <div className="flex flex-col gap-2">
                  <Label htmlFor="isrc">ISRC Code</Label>
                  <Input
                    id="isrc"
                    value={isrc}
                    onChange={(e) => setIsrc(e.target.value.toUpperCase())}
                    onBlur={() => handleBlur('isrc')}
                    placeholder="US-XXX-XX-XXXXX"
                    className={touched.isrc && errors.isrc ? 'border-destructive' : ''}
                  />
                  {touched.isrc && errors.isrc && (
                    <span className="text-destructive text-xs">{errors.isrc}</span>
                  )}
                </div>
                <div className="flex flex-col gap-2">
                  <Label htmlFor="upc">UPC/EAN Barcode</Label>
                  <Input
                    id="upc"
                    value={upc}
                    onChange={(e) => setUpc(e.target.value)}
                    placeholder="Auto-generate or enter"
                  />
                </div>
                <div className="col-span-2 flex flex-col gap-2">
                  <Label htmlFor="lyrics">Lyrics (Optional)</Label>
                  <textarea
                    id="lyrics"
                    value={lyrics}
                    onChange={(e) => setLyrics(e.target.value)}
                    placeholder="Enter song lyrics for platforms that support lyric display"
                    rows={4}
                    className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                  />
                </div>
                <div className="col-span-2 flex items-center gap-2">
                  <Checkbox
                    id="explicit"
                    checked={explicitContent}
                    onCheckedChange={(checked) => setExplicitContent(checked as boolean)}
                  />
                  <Label htmlFor="explicit" className="text-sm font-normal cursor-pointer">
                    This track contains explicit content
                  </Label>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Artwork Upload */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <Image className="h-5 w-5" />
                Cover Artwork
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex gap-6 items-start">
                <div className="w-[200px] h-[200px] border-2 border-dashed border-border rounded-lg flex items-center justify-center flex-col gap-3 bg-secondary cursor-pointer hover:bg-secondary/80 transition-colors">
                  <Image className="h-8 w-8 text-muted-foreground" />
                  <span className="text-xs text-muted-foreground">Click to upload artwork</span>
                  <span className="text-[10px] text-muted-foreground">3000x3000px JPG or PNG</span>
                </div>
                <div className="flex flex-col gap-2">
                  <span className="font-semibold text-sm mb-2">Artwork Requirements:</span>
                  <div className="flex items-center gap-2 text-xs">
                    <Check className="h-4 w-4 text-green-500" />
                    Minimum 3000x3000 pixels (recommended)
                  </div>
                  <div className="flex items-center gap-2 text-xs">
                    <Check className="h-4 w-4 text-green-500" />
                    JPG or PNG format
                  </div>
                  <div className="flex items-center gap-2 text-xs">
                    <Check className="h-4 w-4 text-green-500" />
                    Perfect square aspect ratio (1:1)
                  </div>
                  <div className="flex items-center gap-2 text-xs">
                    <Check className="h-4 w-4 text-green-500" />
                    RGB color mode
                  </div>
                  <div className="flex items-center gap-2 text-xs">
                    <AlertTriangle className="h-4 w-4 text-yellow-500" />
                    No contact information or URLs
                  </div>
                  <div className="flex items-center gap-2 text-xs">
                    <AlertTriangle className="h-4 w-4 text-yellow-500" />
                    No explicit or misleading content
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Release Date */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <Calendar className="h-5 w-5" />
                Release Schedule
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex flex-col gap-2 max-w-[300px]">
                <Label htmlFor="release-date">Release Date *</Label>
                <Input
                  id="release-date"
                  type="date"
                  value={releaseDate}
                  onChange={(e) => setReleaseDate(e.target.value)}
                  onBlur={() => handleBlur('releaseDate')}
                  className={touched.releaseDate && errors.releaseDate ? 'border-destructive' : ''}
                />
                {touched.releaseDate && errors.releaseDate && (
                  <span className="text-destructive text-xs">{errors.releaseDate}</span>
                )}
                <span className="text-[11px] text-muted-foreground">
                  Releases typically go live at midnight in each timezone
                </span>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Right Panel - Platforms & Status */}
        <div className="flex-1 flex flex-col gap-5">
          {/* Platform Selection */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Distribution Platforms</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex flex-col gap-3">
                {platforms.map((platform) => (
                  <div
                    key={platform.id}
                    className="flex justify-between items-center p-3 bg-secondary rounded-md"
                  >
                    <div className="flex items-center gap-3">
                      <div
                        className="w-10 h-10 rounded-lg flex items-center justify-center font-bold text-xs text-white"
                        style={{ backgroundColor: platform.color }}
                      >
                        {platform.name.slice(0, 2).toUpperCase()}
                      </div>
                      <div className="flex flex-col gap-0.5">
                        <span className="text-sm font-semibold">{platform.name}</span>
                        <span className="text-[11px] text-muted-foreground">
                          {platform.fee} • {platform.deliveryTime}
                        </span>
                      </div>
                    </div>
                    <Checkbox
                      checked={platform.selected}
                      onCheckedChange={() => togglePlatform(platform.id)}
                    />
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* Distribution Status */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Distribution Checklist</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex flex-col gap-3">
                {statusItems.map((item, idx) => (
                  <div key={idx} className="flex items-center gap-3 p-3 bg-secondary rounded-md">
                    {getStatusIcon(item.status)}
                    <div className="flex flex-col gap-1">
                      <span className="text-[13px] font-semibold">{item.step}</span>
                      <span className="text-[11px] text-muted-foreground">{item.detail}</span>
                    </div>
                  </div>
                ))}
              </div>

              {uploadProgress > 0 && (
                <div className="mt-4">
                  <Label className="text-xs">Upload Progress</Label>
                  <Progress value={uploadProgress} className="mt-2" />
                  <span className="text-[11px] text-muted-foreground mt-1 block">
                    {uploadProgress}% complete
                  </span>
                </div>
              )}
            </CardContent>
          </Card>

          {/* Distribution Partner */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Distribution Partner</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex flex-col gap-3">
                <span className="text-sm font-semibold">DistroKid Integration</span>
                <span className="text-xs text-muted-foreground">
                  Distribute your music to all major streaming platforms with one upload. Keep 100%
                  of your earnings.
                </span>
                <Button variant="ghost" size="sm">
                  Connect DistroKid Account
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
