/**
 * Distribute Mode - Music distribution platform integration
 * Upload to DistroKid, Spotify for Artists, Apple Music, etc.
 */

import {
  makeStyles,
  shorthands,
  tokens,
  Button,
  Card,
  Input,
  Dropdown,
  Option,
  Textarea,
  Checkbox,
  Label,
  ProgressBar,
} from '@fluentui/react-components';
import {
  CloudArrowUp24Regular,
  MusicNote224Regular,
  Calendar24Regular,
  Image24Regular,
  Checkmark24Regular,
  Warning24Regular,
  FolderOpen24Regular,
} from '@fluentui/react-icons';
import { useState } from 'react';
import { useProject, useAppStore } from '../store/app-store';
import { importFile } from '../services/file-import';

const useStyles = makeStyles({
  container: {
    height: '100%',
    display: 'flex',
    flexDirection: 'column',
  },
  header: {
    ...shorthands.padding('16px'),
    ...shorthands.borderBottom('1px', 'solid', tokens.colorNeutralStroke1),
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  title: {
    fontSize: '20px',
    fontWeight: tokens.fontWeightSemibold,
  },
  content: {
    flex: 1,
    display: 'flex',
    ...shorthands.padding('24px'),
    ...shorthands.gap('24px'),
    overflow: 'auto',
  },
  leftPanel: {
    flex: 2,
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('20px'),
  },
  rightPanel: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('20px'),
  },
  card: {
    ...shorthands.padding('20px'),
  },
  cardTitle: {
    fontSize: '16px',
    fontWeight: tokens.fontWeightSemibold,
    marginBottom: '16px',
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
  },
  formGrid: {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    ...shorthands.gap('16px'),
  },
  formField: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
  },
  formFieldFull: {
    gridColumn: '1 / -1',
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
  },
  platformList: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('12px'),
  },
  platformItem: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    ...shorthands.padding('12px'),
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.borderRadius('6px'),
  },
  platformInfo: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('12px'),
  },
  platformLogo: {
    width: '40px',
    height: '40px',
    ...shorthands.borderRadius('8px'),
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontWeight: 700,
    fontSize: '12px',
    color: 'white',
  },
  platformDetails: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('2px'),
  },
  platformName: {
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
  },
  platformStatus: {
    fontSize: '11px',
    color: tokens.colorNeutralForeground2,
  },
  statusList: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('12px'),
  },
  statusItem: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('12px'),
    ...shorthands.padding('12px'),
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.borderRadius('6px'),
  },
  statusIcon: {
    width: '24px',
    height: '24px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  statusText: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('4px'),
  },
  statusTitle: {
    fontSize: '13px',
    fontWeight: tokens.fontWeightSemibold,
  },
  statusDetail: {
    fontSize: '11px',
    color: tokens.colorNeutralForeground2,
  },
  artworkUpload: {
    width: '200px',
    height: '200px',
    ...shorthands.border('2px', 'dashed', tokens.colorNeutralStroke1),
    ...shorthands.borderRadius('8px'),
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexDirection: 'column',
    ...shorthands.gap('12px'),
    backgroundColor: tokens.colorNeutralBackground3,
    cursor: 'pointer',
    transition: 'all 0.2s ease',
  },
  requirementsList: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
  },
  requirementItem: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
    fontSize: '12px',
  },
  emptyState: {
    flex: 1,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexDirection: 'column',
    ...shorthands.gap('16px'),
  },
});

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
  const styles = useStyles();
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

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    // Required fields
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

    // ISRC format validation (optional but validated if provided)
    if (isrc && !/^[A-Z]{2}-[A-Z0-9]{3}-\d{2}-\d{5}$/.test(isrc.toUpperCase())) {
      newErrors.isrc = 'Invalid ISRC format (example: US-ABC-12-34567)';
    }

    // Date validation
    if (releaseDate) {
      const date = new Date(releaseDate);
      const today = new Date();
      today.setHours(0, 0, 0, 0);
      if (date < today) {
        newErrors.releaseDate = 'Release date must be today or in the future';
      }
    }

    // Platform selection
    if (selectedPlatformCount === 0) {
      newErrors.platforms = 'Please select at least one distribution platform';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = () => {
    // Mark all fields as touched
    setTouched({
      trackTitle: true,
      artistName: true,
      genre: true,
      releaseDate: true,
      platforms: true,
    });

    if (!validateForm()) {
      return; // Don't submit if validation fails
    }

    // Simulate upload progress
    let progress = 0;
    const interval = setInterval(() => {
      progress += 10;
      setUploadProgress(progress);
      if (progress >= 100) {
        clearInterval(interval);
        console.log('[Distribute] Submission complete');
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
        return <Checkmark24Regular color={tokens.colorPaletteGreenForeground1} />;
      case 'warning':
        return <Warning24Regular color={tokens.colorPaletteYellowForeground1} />;
      case 'in_progress':
        return <ProgressBar />;
      default:
        return <div style={{ width: '24px', height: '24px' }} />;
    }
  };

  const selectedPlatformCount = platforms.filter((p) => p.selected).length;

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
      <div className={styles.container}>
        <div className={styles.header}>
          <div className={styles.title}>🌍 Distribute - Music Distribution</div>
        </div>
        <div className={styles.emptyState}>
          <h3>No Project Loaded</h3>
          <p style={{ color: tokens.colorNeutralForeground2, marginBottom: '24px' }}>
            To use Distribute mode, you need to load a project first.
          </p>
          <div style={{ display: 'flex', gap: '12px', justifyContent: 'center' }}>
            <Button
              appearance="primary"
              icon={<FolderOpen24Regular />}
              onClick={handleImportFile}
            >
              Import Guitar Pro File
            </Button>
            <Button
              appearance="secondary"
              onClick={() => setMode('compose')}
            >
              Go to Compose Mode
            </Button>
          </div>
          <p style={{ fontSize: '12px', color: tokens.colorNeutralForeground3, marginTop: '16px' }}>
            Or press <strong>Ctrl+1</strong> to switch to Compose mode
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.title}>🌍 Distribute - Music Distribution</div>
        <Button
          icon={<CloudArrowUp24Regular />}
          appearance="primary"
          onClick={handleSubmit}
          disabled={selectedPlatformCount === 0}
        >
          Submit to {selectedPlatformCount} {selectedPlatformCount === 1 ? 'Platform' : 'Platforms'}
        </Button>
        {touched.platforms && errors.platforms && (
          <div style={{ color: tokens.colorPaletteRedForeground1, fontSize: '12px', marginLeft: '8px' }}>
            {errors.platforms}
          </div>
        )}
      </div>

      <div className={styles.content}>
        {/* Left Panel - Metadata & Artwork */}
        <div className={styles.leftPanel}>
          {/* Release Information */}
          <Card className={styles.card}>
            <div className={styles.cardTitle}>
              <MusicNote224Regular />
              Release Information
            </div>
            <div className={styles.formGrid}>
              <div className={styles.formField}>
                <Label htmlFor="track-title" required>
                  Track Title
                </Label>
                <Input
                  id="track-title"
                  value={trackTitle}
                  onChange={(e) => setTrackTitle(e.target.value)}
                  onBlur={() => handleBlur('trackTitle')}
                  placeholder="Enter track title"
                  style={touched.trackTitle && errors.trackTitle ? { borderColor: tokens.colorPaletteRedBorder1 } : {}}
                />
                {touched.trackTitle && errors.trackTitle && (
                  <div style={{ color: tokens.colorPaletteRedForeground1, fontSize: '12px', marginTop: '4px' }}>
                    {errors.trackTitle}
                  </div>
                )}
              </div>
              <div className={styles.formField}>
                <Label htmlFor="artist-name" required>
                  Artist Name
                </Label>
                <Input
                  id="artist-name"
                  value={artistName}
                  onChange={(e) => setArtistName(e.target.value)}
                  onBlur={() => handleBlur('artistName')}
                  placeholder="Enter artist name"
                  style={touched.artistName && errors.artistName ? { borderColor: tokens.colorPaletteRedBorder1 } : {}}
                />
                {touched.artistName && errors.artistName && (
                  <div style={{ color: tokens.colorPaletteRedForeground1, fontSize: '12px', marginTop: '4px' }}>
                    {errors.artistName}
                  </div>
                )}
              </div>
              <div className={styles.formField}>
                <Label htmlFor="album-title">Album Title</Label>
                <Input
                  id="album-title"
                  value={albumTitle}
                  onChange={(e) => setAlbumTitle(e.target.value)}
                  placeholder="Single, EP, or Album name"
                />
              </div>
              <div className={styles.formField}>
                <Label htmlFor="genre" required>
                  Genre
                </Label>
                <Dropdown
                  id="genre"
                  placeholder="Select genre"
                  value={genre}
                  onOptionSelect={(_, data) => {
                    setGenre(data.optionValue as string);
                    handleBlur('genre');
                  }}
                  style={touched.genre && errors.genre ? { borderColor: tokens.colorPaletteRedBorder1 } : {}}
                >
                  <Option value="rock" text="Rock">Rock</Option>
                  <Option value="metal" text="Metal">Metal</Option>
                  <Option value="pop" text="Pop">Pop</Option>
                  <Option value="electronic" text="Electronic">Electronic</Option>
                  <Option value="hip-hop" text="Hip Hop">Hip Hop</Option>
                  <Option value="jazz" text="Jazz">Jazz</Option>
                  <Option value="classical" text="Classical">Classical</Option>
                  <Option value="country" text="Country">Country</Option>
                </Dropdown>
                {touched.genre && errors.genre && (
                  <div style={{ color: tokens.colorPaletteRedForeground1, fontSize: '12px', marginTop: '4px' }}>
                    {errors.genre}
                  </div>
                )}
              </div>
              <div className={styles.formField}>
                <Label htmlFor="isrc">ISRC Code</Label>
                <Input
                  id="isrc"
                  value={isrc}
                  onChange={(e) => setIsrc(e.target.value.toUpperCase())}
                  onBlur={() => handleBlur('isrc')}
                  placeholder="US-XXX-XX-XXXXX"
                  style={touched.isrc && errors.isrc ? { borderColor: tokens.colorPaletteRedBorder1 } : {}}
                />
                {touched.isrc && errors.isrc && (
                  <div style={{ color: tokens.colorPaletteRedForeground1, fontSize: '12px', marginTop: '4px' }}>
                    {errors.isrc}
                  </div>
                )}
              </div>
              <div className={styles.formField}>
                <Label htmlFor="upc">UPC/EAN Barcode</Label>
                <Input
                  id="upc"
                  value={upc}
                  onChange={(e) => setUpc(e.target.value)}
                  placeholder="Auto-generate or enter"
                />
              </div>
              <div className={styles.formFieldFull}>
                <Label htmlFor="lyrics">Lyrics (Optional)</Label>
                <Textarea
                  id="lyrics"
                  value={lyrics}
                  onChange={(e) => setLyrics(e.target.value)}
                  placeholder="Enter song lyrics for platforms that support lyric display"
                  rows={4}
                />
              </div>
              <div className={styles.formFieldFull}>
                <Checkbox
                  checked={explicitContent}
                  onChange={(_, data) => setExplicitContent(data.checked as boolean)}
                  label="This track contains explicit content"
                />
              </div>
            </div>
          </Card>

          {/* Artwork Upload */}
          <Card className={styles.card}>
            <div className={styles.cardTitle}>
              <Image24Regular />
              Cover Artwork
            </div>
            <div style={{ display: 'flex', gap: '24px', alignItems: 'flex-start' }}>
              <div className={styles.artworkUpload}>
                <Image24Regular fontSize={32} color={tokens.colorNeutralForeground2} />
                <div style={{ fontSize: '12px', color: tokens.colorNeutralForeground2 }}>
                  Click to upload artwork
                </div>
                <div style={{ fontSize: '10px', color: tokens.colorNeutralForeground3 }}>
                  3000x3000px JPG or PNG
                </div>
              </div>
              <div className={styles.requirementsList}>
                <div style={{ fontWeight: tokens.fontWeightSemibold, marginBottom: '8px' }}>
                  Artwork Requirements:
                </div>
                <div className={styles.requirementItem}>
                  <Checkmark24Regular fontSize={16} color={tokens.colorPaletteGreenForeground1} />
                  Minimum 3000x3000 pixels (recommended)
                </div>
                <div className={styles.requirementItem}>
                  <Checkmark24Regular fontSize={16} color={tokens.colorPaletteGreenForeground1} />
                  JPG or PNG format
                </div>
                <div className={styles.requirementItem}>
                  <Checkmark24Regular fontSize={16} color={tokens.colorPaletteGreenForeground1} />
                  Perfect square aspect ratio (1:1)
                </div>
                <div className={styles.requirementItem}>
                  <Checkmark24Regular fontSize={16} color={tokens.colorPaletteGreenForeground1} />
                  RGB color mode
                </div>
                <div className={styles.requirementItem}>
                  <Warning24Regular fontSize={16} color={tokens.colorPaletteYellowForeground1} />
                  No contact information or URLs
                </div>
                <div className={styles.requirementItem}>
                  <Warning24Regular fontSize={16} color={tokens.colorPaletteYellowForeground1} />
                  No explicit or misleading content
                </div>
              </div>
            </div>
          </Card>

          {/* Release Date */}
          <Card className={styles.card}>
            <div className={styles.cardTitle}>
              <Calendar24Regular />
              Release Schedule
            </div>
            <div className={styles.formField}>
              <Label htmlFor="release-date" required>
                Release Date
              </Label>
              <Input
                id="release-date"
                type="date"
                value={releaseDate}
                onChange={(e) => setReleaseDate(e.target.value)}
                onBlur={() => handleBlur('releaseDate')}
                style={touched.releaseDate && errors.releaseDate ? { borderColor: tokens.colorPaletteRedBorder1 } : {}}
              />
              {touched.releaseDate && errors.releaseDate && (
                <div style={{ color: tokens.colorPaletteRedForeground1, fontSize: '12px', marginTop: '4px' }}>
                  {errors.releaseDate}
                </div>
              )}
              <div style={{ fontSize: '11px', color: tokens.colorNeutralForeground2, marginTop: '4px' }}>
                Releases typically go live at midnight in each timezone
              </div>
            </div>
          </Card>
        </div>

        {/* Right Panel - Platforms & Status */}
        <div className={styles.rightPanel}>
          {/* Platform Selection */}
          <Card className={styles.card}>
            <div className={styles.cardTitle}>Distribution Platforms</div>
            <div className={styles.platformList}>
              {platforms.map((platform) => (
                <div key={platform.id} className={styles.platformItem}>
                  <div className={styles.platformInfo}>
                    <div
                      className={styles.platformLogo}
                      style={{ backgroundColor: platform.color }}
                    >
                      {platform.name.slice(0, 2).toUpperCase()}
                    </div>
                    <div className={styles.platformDetails}>
                      <div className={styles.platformName}>{platform.name}</div>
                      <div className={styles.platformStatus}>
                        {platform.fee} • {platform.deliveryTime}
                      </div>
                    </div>
                  </div>
                  <Checkbox
                    checked={platform.selected}
                    onChange={() => togglePlatform(platform.id)}
                  />
                </div>
              ))}
            </div>
          </Card>

          {/* Distribution Status */}
          <Card className={styles.card}>
            <div className={styles.cardTitle}>Distribution Checklist</div>
            <div className={styles.statusList}>
              {statusItems.map((item, idx) => (
                <div key={idx} className={styles.statusItem}>
                  <div className={styles.statusIcon}>{getStatusIcon(item.status)}</div>
                  <div className={styles.statusText}>
                    <div className={styles.statusTitle}>{item.step}</div>
                    <div className={styles.statusDetail}>{item.detail}</div>
                  </div>
                </div>
              ))}
            </div>

            {uploadProgress > 0 && (
              <div style={{ marginTop: '16px' }}>
                <Label>Upload Progress</Label>
                <ProgressBar value={uploadProgress / 100} />
                <div
                  style={{
                    fontSize: '11px',
                    color: tokens.colorNeutralForeground2,
                    marginTop: '4px',
                  }}
                >
                  {uploadProgress}% complete
                </div>
              </div>
            )}
          </Card>

          {/* Distribution Partner */}
          <Card className={styles.card}>
            <div className={styles.cardTitle}>Distribution Partner</div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
              <div style={{ fontSize: '14px', fontWeight: tokens.fontWeightSemibold }}>
                DistroKid Integration
              </div>
              <div style={{ fontSize: '12px', color: tokens.colorNeutralForeground2 }}>
                Distribute your music to all major streaming platforms with one upload. Keep 100%
                of your earnings.
              </div>
              <Button appearance="subtle" size="small">
                Connect DistroKid Account
              </Button>
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}
