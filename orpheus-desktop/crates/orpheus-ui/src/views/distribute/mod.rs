//! Distribute mode view - release management and distribution

use egui::{Color32, Ui, ScrollArea, Vec2, Stroke};
use crate::theme::Theme;
use std::path::PathBuf;

// ============================================================================
// Release Metadata - Track and Album Information
// ============================================================================

/// Complete release metadata for distribution
#[derive(Clone)]
pub struct ReleaseMetadata {
    /// Track/Single title
    pub title: String,
    /// Primary artist name
    pub artist: String,
    /// Album name (if part of album)
    pub album: String,
    /// Album artist (if different from track artist)
    pub album_artist: String,
    /// Genre
    pub genre: String,
    /// Sub-genre
    pub subgenre: String,
    /// Release date (YYYY-MM-DD)
    pub release_date: String,
    /// Year
    pub year: u16,
    /// Track number
    pub track_number: u16,
    /// Total tracks in album
    pub total_tracks: u16,
    /// Disc number
    pub disc_number: u16,
    /// Total discs
    pub total_discs: u16,
    /// ISRC (International Standard Recording Code)
    pub isrc: String,
    /// UPC/EAN (Universal Product Code)
    pub upc: String,
    /// Composer(s)
    pub composer: String,
    /// Lyricist(s)
    pub lyricist: String,
    /// Producer(s)
    pub producer: String,
    /// Record label
    pub label: String,
    /// Copyright notice
    pub copyright: String,
    /// Copyright year
    pub copyright_year: u16,
    /// Publishing rights (P line)
    pub p_line: String,
    /// Is explicit content
    pub explicit: bool,
    /// Language (ISO 639-1)
    pub language: String,
    /// Lyrics
    pub lyrics: String,
    /// BPM
    pub bpm: u16,
    /// Key signature
    pub key: String,
    /// Additional comments/notes
    pub comments: String,
}

impl Default for ReleaseMetadata {
    fn default() -> Self {
        let current_year = 2024u16; // Would use actual year in production

        Self {
            title: String::new(),
            artist: String::new(),
            album: String::new(),
            album_artist: String::new(),
            genre: "Rock".to_string(),
            subgenre: String::new(),
            release_date: String::new(),
            year: current_year,
            track_number: 1,
            total_tracks: 1,
            disc_number: 1,
            total_discs: 1,
            isrc: String::new(),
            upc: String::new(),
            composer: String::new(),
            lyricist: String::new(),
            producer: String::new(),
            label: String::new(),
            copyright: String::new(),
            copyright_year: current_year,
            p_line: String::new(),
            explicit: false,
            language: "en".to_string(),
            lyrics: String::new(),
            bpm: 120,
            key: String::new(),
            comments: String::new(),
        }
    }
}

impl ReleaseMetadata {
    /// Check if required fields are filled
    pub fn is_complete(&self) -> bool {
        !self.title.is_empty()
            && !self.artist.is_empty()
            && !self.genre.is_empty()
    }

    /// Check if ready for distribution (all recommended fields)
    pub fn is_distribution_ready(&self) -> bool {
        self.is_complete()
            && !self.isrc.is_empty()
            && !self.release_date.is_empty()
            && !self.copyright.is_empty()
    }

    /// Get completion percentage
    pub fn completion_percentage(&self) -> u8 {
        let mut filled = 0u8;
        let total = 10u8; // Core fields

        if !self.title.is_empty() { filled += 1; }
        if !self.artist.is_empty() { filled += 1; }
        if !self.album.is_empty() { filled += 1; }
        if !self.genre.is_empty() { filled += 1; }
        if !self.release_date.is_empty() { filled += 1; }
        if !self.isrc.is_empty() { filled += 1; }
        if !self.copyright.is_empty() { filled += 1; }
        if !self.label.is_empty() { filled += 1; }
        if !self.composer.is_empty() { filled += 1; }
        if !self.language.is_empty() { filled += 1; }

        ((filled as f32 / total as f32) * 100.0) as u8
    }
}

// ============================================================================
// Cover Art - Album Artwork Management
// ============================================================================

/// Cover art configuration
#[derive(Clone)]
pub struct CoverArt {
    /// Path to cover art file
    pub path: Option<PathBuf>,
    /// Image dimensions (width, height)
    pub dimensions: Option<(u32, u32)>,
    /// File size in bytes
    pub file_size: Option<u64>,
    /// Is valid for distribution (3000x3000 minimum, square)
    pub is_valid: bool,
    /// Validation messages
    pub validation_messages: Vec<String>,
}

impl Default for CoverArt {
    fn default() -> Self {
        Self {
            path: None,
            dimensions: None,
            file_size: None,
            is_valid: false,
            validation_messages: vec!["No cover art selected".to_string()],
        }
    }
}

impl CoverArt {
    /// Validate cover art for distribution
    pub fn validate(&mut self) {
        self.validation_messages.clear();
        self.is_valid = true;

        if self.path.is_none() {
            self.validation_messages.push("No cover art selected".to_string());
            self.is_valid = false;
            return;
        }

        if let Some((w, h)) = self.dimensions {
            // Check minimum size (3000x3000 for most platforms)
            if w < 3000 || h < 3000 {
                self.validation_messages.push(format!(
                    "Image too small: {}x{} (minimum 3000x3000)",
                    w, h
                ));
                self.is_valid = false;
            }

            // Check square aspect ratio
            if w != h {
                self.validation_messages.push(format!(
                    "Image not square: {}x{} (must be 1:1 ratio)",
                    w, h
                ));
                self.is_valid = false;
            }

            // Check maximum size
            if w > 10000 || h > 10000 {
                self.validation_messages.push(format!(
                    "Image too large: {}x{} (maximum 10000x10000)",
                    w, h
                ));
                self.is_valid = false;
            }
        }

        // Check file size (max 20MB for most platforms)
        if let Some(size) = self.file_size {
            if size > 20 * 1024 * 1024 {
                self.validation_messages.push(format!(
                    "File too large: {:.1}MB (maximum 20MB)",
                    size as f64 / (1024.0 * 1024.0)
                ));
                self.is_valid = false;
            }
        }

        if self.is_valid {
            self.validation_messages.push("Cover art meets all requirements".to_string());
        }
    }
}

// ============================================================================
// Distribution Platforms - Target Services
// ============================================================================

/// Distribution platform target
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DistributionPlatform {
    Spotify,
    AppleMusic,
    YouTube,
    YouTubeMusic,
    AmazonMusic,
    Deezer,
    Tidal,
    Pandora,
    SoundCloud,
    Bandcamp,
    TikTok,
    Instagram,
    Facebook,
}

impl DistributionPlatform {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Spotify => "Spotify",
            Self::AppleMusic => "Apple Music",
            Self::YouTube => "YouTube",
            Self::YouTubeMusic => "YouTube Music",
            Self::AmazonMusic => "Amazon Music",
            Self::Deezer => "Deezer",
            Self::Tidal => "Tidal",
            Self::Pandora => "Pandora",
            Self::SoundCloud => "SoundCloud",
            Self::Bandcamp => "Bandcamp",
            Self::TikTok => "TikTok",
            Self::Instagram => "Instagram",
            Self::Facebook => "Facebook",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Spotify => "🎵",
            Self::AppleMusic => "🍎",
            Self::YouTube | Self::YouTubeMusic => "▶",
            Self::AmazonMusic => "📦",
            Self::Deezer => "🎧",
            Self::Tidal => "🌊",
            Self::Pandora => "📻",
            Self::SoundCloud => "☁",
            Self::Bandcamp => "🎸",
            Self::TikTok => "🎬",
            Self::Instagram => "📷",
            Self::Facebook => "👤",
        }
    }

    pub fn category(&self) -> PlatformCategory {
        match self {
            Self::Spotify | Self::AppleMusic | Self::AmazonMusic |
            Self::Deezer | Self::Tidal | Self::Pandora | Self::YouTubeMusic => PlatformCategory::Streaming,
            Self::YouTube => PlatformCategory::Video,
            Self::SoundCloud | Self::Bandcamp => PlatformCategory::Independent,
            Self::TikTok | Self::Instagram | Self::Facebook => PlatformCategory::Social,
        }
    }

    pub fn requirements(&self) -> &'static str {
        match self {
            Self::Spotify => "ISRC required, -14 LUFS normalized",
            Self::AppleMusic => "ISRC required, -16 LUFS normalized",
            Self::YouTube | Self::YouTubeMusic => "Content ID registration available",
            Self::AmazonMusic => "ISRC required, UPC for albums",
            Self::Deezer => "ISRC required",
            Self::Tidal => "ISRC required, MQA optional",
            Self::Pandora => "US-focused, ISRC required",
            Self::SoundCloud => "Direct upload, no ISRC required",
            Self::Bandcamp => "Direct upload, fan-supported",
            Self::TikTok => "15-60 second clips",
            Self::Instagram | Self::Facebook => "Integrated with Meta",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Spotify, Self::AppleMusic, Self::YouTube, Self::YouTubeMusic,
            Self::AmazonMusic, Self::Deezer, Self::Tidal, Self::Pandora,
            Self::SoundCloud, Self::Bandcamp, Self::TikTok, Self::Instagram, Self::Facebook,
        ]
    }
}

/// Platform category
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlatformCategory {
    Streaming,
    Video,
    Independent,
    Social,
}

impl PlatformCategory {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Streaming => "Streaming Services",
            Self::Video => "Video Platforms",
            Self::Independent => "Independent",
            Self::Social => "Social Media",
        }
    }
}

/// Platform selection state
#[derive(Clone)]
pub struct PlatformSelection {
    pub platform: DistributionPlatform,
    pub enabled: bool,
    pub status: PlatformStatus,
}

/// Platform distribution status
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlatformStatus {
    NotSelected,
    Pending,
    Uploading,
    Processing,
    Live,
    Failed,
}

impl PlatformStatus {
    pub fn color(&self) -> Color32 {
        match self {
            Self::NotSelected => Color32::from_gray(100),
            Self::Pending => Color32::from_rgb(241, 196, 15),
            Self::Uploading => Color32::from_rgb(52, 152, 219),
            Self::Processing => Color32::from_rgb(155, 89, 182),
            Self::Live => Color32::from_rgb(46, 204, 113),
            Self::Failed => Color32::from_rgb(231, 76, 60),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::NotSelected => "Not Selected",
            Self::Pending => "Pending",
            Self::Uploading => "Uploading",
            Self::Processing => "Processing",
            Self::Live => "Live",
            Self::Failed => "Failed",
        }
    }
}

// ============================================================================
// Release Checklist - Validation Steps
// ============================================================================

/// Checklist item
#[derive(Clone)]
pub struct ChecklistItem {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub is_required: bool,
    pub is_complete: bool,
    pub category: ChecklistCategory,
}

/// Checklist category
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChecklistCategory {
    Metadata,
    Audio,
    Artwork,
    Legal,
}

impl ChecklistCategory {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Metadata => "Metadata",
            Self::Audio => "Audio",
            Self::Artwork => "Artwork",
            Self::Legal => "Legal",
        }
    }
}

/// Pre-release checklist
#[derive(Clone)]
pub struct ReleaseChecklist {
    pub items: Vec<ChecklistItem>,
}

impl Default for ReleaseChecklist {
    fn default() -> Self {
        Self {
            items: vec![
                // Metadata
                ChecklistItem {
                    id: "title",
                    label: "Track Title",
                    description: "Enter the official track title",
                    is_required: true,
                    is_complete: false,
                    category: ChecklistCategory::Metadata,
                },
                ChecklistItem {
                    id: "artist",
                    label: "Artist Name",
                    description: "Primary artist or band name",
                    is_required: true,
                    is_complete: false,
                    category: ChecklistCategory::Metadata,
                },
                ChecklistItem {
                    id: "genre",
                    label: "Genre",
                    description: "Select primary genre",
                    is_required: true,
                    is_complete: false,
                    category: ChecklistCategory::Metadata,
                },
                ChecklistItem {
                    id: "isrc",
                    label: "ISRC Code",
                    description: "International Standard Recording Code",
                    is_required: true,
                    is_complete: false,
                    category: ChecklistCategory::Metadata,
                },
                ChecklistItem {
                    id: "release_date",
                    label: "Release Date",
                    description: "Scheduled release date",
                    is_required: true,
                    is_complete: false,
                    category: ChecklistCategory::Metadata,
                },
                // Audio
                ChecklistItem {
                    id: "master_export",
                    label: "Master Exported",
                    description: "High-quality master file exported",
                    is_required: true,
                    is_complete: false,
                    category: ChecklistCategory::Audio,
                },
                ChecklistItem {
                    id: "loudness_check",
                    label: "Loudness Check",
                    description: "LUFS levels meet platform requirements",
                    is_required: true,
                    is_complete: false,
                    category: ChecklistCategory::Audio,
                },
                ChecklistItem {
                    id: "quality_check",
                    label: "Quality Check",
                    description: "No clipping, artifacts, or issues",
                    is_required: true,
                    is_complete: false,
                    category: ChecklistCategory::Audio,
                },
                // Artwork
                ChecklistItem {
                    id: "cover_art",
                    label: "Cover Art",
                    description: "3000x3000 square image",
                    is_required: true,
                    is_complete: false,
                    category: ChecklistCategory::Artwork,
                },
                ChecklistItem {
                    id: "cover_valid",
                    label: "Cover Validated",
                    description: "Meets platform requirements",
                    is_required: true,
                    is_complete: false,
                    category: ChecklistCategory::Artwork,
                },
                // Legal
                ChecklistItem {
                    id: "copyright",
                    label: "Copyright Info",
                    description: "Copyright notice and year",
                    is_required: true,
                    is_complete: false,
                    category: ChecklistCategory::Legal,
                },
                ChecklistItem {
                    id: "rights",
                    label: "Rights Confirmed",
                    description: "You own or have rights to distribute",
                    is_required: true,
                    is_complete: false,
                    category: ChecklistCategory::Legal,
                },
            ],
        }
    }
}

impl ReleaseChecklist {
    /// Update checklist based on metadata and state
    pub fn update(&mut self, metadata: &ReleaseMetadata, cover: &CoverArt, has_master: bool) {
        for item in &mut self.items {
            item.is_complete = match item.id {
                "title" => !metadata.title.is_empty(),
                "artist" => !metadata.artist.is_empty(),
                "genre" => !metadata.genre.is_empty(),
                "isrc" => !metadata.isrc.is_empty(),
                "release_date" => !metadata.release_date.is_empty(),
                "master_export" => has_master,
                "loudness_check" => has_master, // Would check actual levels
                "quality_check" => has_master,  // Would run quality analysis
                "cover_art" => cover.path.is_some(),
                "cover_valid" => cover.is_valid,
                "copyright" => !metadata.copyright.is_empty(),
                "rights" => true, // User must confirm
                _ => false,
            };
        }
    }

    /// Get completion stats
    pub fn completion_stats(&self) -> (usize, usize) {
        let required: Vec<_> = self.items.iter().filter(|i| i.is_required).collect();
        let complete = required.iter().filter(|i| i.is_complete).count();
        (complete, required.len())
    }

    /// Is ready for distribution
    pub fn is_ready(&self) -> bool {
        self.items.iter()
            .filter(|i| i.is_required)
            .all(|i| i.is_complete)
    }
}

// ============================================================================
// Promotional Materials - Marketing Assets
// ============================================================================

/// Promotional material type
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PromoMaterialType {
    SocialSquare,    // 1080x1080 for Instagram
    SocialStory,     // 1080x1920 for Stories
    Banner,          // 1500x500 for Twitter/X
    YouTubeThumbnail, // 1280x720
    PressKit,        // PDF press kit
    BioText,         // Artist bio
    EPK,             // Electronic Press Kit
}

impl PromoMaterialType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SocialSquare => "Social Square (1080x1080)",
            Self::SocialStory => "Social Story (1080x1920)",
            Self::Banner => "Banner (1500x500)",
            Self::YouTubeThumbnail => "YouTube Thumbnail",
            Self::PressKit => "Press Kit (PDF)",
            Self::BioText => "Artist Bio",
            Self::EPK => "Electronic Press Kit",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::SocialSquare => "Square image for Instagram, Facebook posts",
            Self::SocialStory => "Vertical image for Instagram/TikTok stories",
            Self::Banner => "Wide banner for Twitter/X, YouTube",
            Self::YouTubeThumbnail => "Thumbnail for YouTube videos",
            Self::PressKit => "PDF with release info for press",
            Self::BioText => "Short and long bio text",
            Self::EPK => "Complete electronic press kit",
        }
    }
}

/// Promotional material
#[derive(Clone)]
pub struct PromoMaterial {
    pub material_type: PromoMaterialType,
    pub generated: bool,
    pub path: Option<PathBuf>,
}

// ============================================================================
// Distribute View State
// ============================================================================

/// Active tab in distribute view
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DistributeTab {
    Overview,
    Metadata,
    Artwork,
    Platforms,
    Checklist,
    Promo,
}

/// State for the distribute view
pub struct DistributeViewState {
    /// Release metadata
    pub metadata: ReleaseMetadata,
    /// Cover art
    pub cover_art: CoverArt,
    /// Platform selections
    pub platforms: Vec<PlatformSelection>,
    /// Release checklist
    pub checklist: ReleaseChecklist,
    /// Promotional materials
    pub promo_materials: Vec<PromoMaterial>,
    /// Has exported master file
    pub has_master: bool,
    /// Master file path
    pub master_path: Option<PathBuf>,
    /// Master file format info
    pub master_info: Option<MasterFileInfo>,
    /// Active tab
    pub active_tab: DistributeTab,
    /// Show advanced metadata
    pub show_advanced_metadata: bool,
    /// Distribution in progress
    pub distribution_in_progress: bool,
    /// Selected metadata section
    pub metadata_section: MetadataSection,
}

/// Master file info
#[derive(Clone)]
pub struct MasterFileInfo {
    pub format: String,
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub duration_secs: f32,
    pub lufs: f32,
    pub true_peak: f32,
}

/// Metadata editor section
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MetadataSection {
    Basic,
    Album,
    Credits,
    Codes,
    Legal,
}

impl Default for DistributeViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl DistributeViewState {
    pub fn new() -> Self {
        // Initialize platform selections
        let platforms = DistributionPlatform::all()
            .into_iter()
            .map(|p| PlatformSelection {
                platform: p,
                enabled: matches!(p,
                    DistributionPlatform::Spotify |
                    DistributionPlatform::AppleMusic |
                    DistributionPlatform::YouTube |
                    DistributionPlatform::AmazonMusic
                ),
                status: PlatformStatus::NotSelected,
            })
            .collect();

        // Initialize promo materials
        let promo_materials = vec![
            PromoMaterial { material_type: PromoMaterialType::SocialSquare, generated: false, path: None },
            PromoMaterial { material_type: PromoMaterialType::SocialStory, generated: false, path: None },
            PromoMaterial { material_type: PromoMaterialType::Banner, generated: false, path: None },
            PromoMaterial { material_type: PromoMaterialType::YouTubeThumbnail, generated: false, path: None },
        ];

        Self {
            metadata: ReleaseMetadata::default(),
            cover_art: CoverArt::default(),
            platforms,
            checklist: ReleaseChecklist::default(),
            promo_materials,
            has_master: false,
            master_path: None,
            master_info: None,
            active_tab: DistributeTab::Overview,
            show_advanced_metadata: false,
            distribution_in_progress: false,
            metadata_section: MetadataSection::Basic,
        }
    }

    /// Update checklist from current state
    pub fn update_checklist(&mut self) {
        self.checklist.update(&self.metadata, &self.cover_art, self.has_master);
    }

    /// Get enabled platform count
    pub fn enabled_platform_count(&self) -> usize {
        self.platforms.iter().filter(|p| p.enabled).count()
    }

    /// Get overall readiness percentage
    pub fn readiness_percentage(&self) -> u8 {
        let (complete, total) = self.checklist.completion_stats();
        if total == 0 { return 0; }
        ((complete as f32 / total as f32) * 100.0) as u8
    }
}

// ============================================================================
// Distribute View Component
// ============================================================================

/// Distribute mode view
pub struct DistributeView<'a> {
    state: &'a mut DistributeViewState,
    theme: &'a Theme,
}

impl<'a> DistributeView<'a> {
    pub fn new(state: &'a mut DistributeViewState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        // Update checklist
        self.state.update_checklist();

        // Handle keyboard
        self.handle_keyboard(ui);

        ui.vertical(|ui| {
            // Header with tabs
            self.show_header(ui);

            ui.separator();

            // Main content based on active tab
            match self.state.active_tab {
                DistributeTab::Overview => self.show_overview_tab(ui),
                DistributeTab::Metadata => self.show_metadata_tab(ui),
                DistributeTab::Artwork => self.show_artwork_tab(ui),
                DistributeTab::Platforms => self.show_platforms_tab(ui),
                DistributeTab::Checklist => self.show_checklist_tab(ui),
                DistributeTab::Promo => self.show_promo_tab(ui),
            }

            // Status bar
            ui.separator();
            self.show_status_bar(ui);
        });
    }

    fn show_header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Distribution");

            ui.separator();

            // Tab buttons
            let tabs = [
                (DistributeTab::Overview, "Overview", "Release overview (1)"),
                (DistributeTab::Metadata, "Metadata", "Track information (2)"),
                (DistributeTab::Artwork, "Artwork", "Cover art (3)"),
                (DistributeTab::Platforms, "Platforms", "Distribution targets (4)"),
                (DistributeTab::Checklist, "Checklist", "Pre-release checks (5)"),
                (DistributeTab::Promo, "Promo", "Promotional materials (6)"),
            ];

            for (tab, label, tooltip) in tabs {
                let is_selected = self.state.active_tab == tab;
                if ui.selectable_label(is_selected, label)
                    .on_hover_text(tooltip)
                    .clicked()
                {
                    self.state.active_tab = tab;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Readiness indicator
                let readiness = self.state.readiness_percentage();
                let readiness_color = if readiness >= 100 {
                    Color32::from_rgb(46, 204, 113)
                } else if readiness >= 70 {
                    Color32::from_rgb(241, 196, 15)
                } else {
                    Color32::from_rgb(231, 76, 60)
                };

                ui.label(egui::RichText::new(format!("{}% Ready", readiness))
                    .color(readiness_color)
                    .strong());
            });
        });
    }

    fn show_overview_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Left side - Release card
            ui.allocate_ui_with_layout(
                Vec2::new(300.0, ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    self.show_release_card(ui);
                },
            );

            ui.separator();

            // Right side - Quick stats and actions
            ui.vertical(|ui| {
                // Quick stats
                self.show_quick_stats(ui);

                ui.add_space(16.0);

                // Quick actions
                self.show_quick_actions(ui);

                ui.add_space(16.0);

                // Platform summary
                self.show_platform_summary(ui);
            });
        });
    }

    fn show_release_card(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            // Cover art preview
            let cover_size = 200.0;
            let (rect, _) = ui.allocate_exact_size(Vec2::new(cover_size, cover_size), egui::Sense::click());

            if let Some(_path) = &self.state.cover_art.path {
                // Would display actual image
                ui.painter().rect_filled(rect, 8.0, Color32::from_rgb(60, 60, 60));
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Cover Art",
                    egui::FontId::default(),
                    Color32::WHITE,
                );
            } else {
                ui.painter().rect_filled(rect, 8.0, self.theme.palette.bg_tertiary);
                ui.painter().rect_stroke(rect, 8.0, Stroke::new(2.0, Color32::from_gray(80)));
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "No Cover Art\nClick to add",
                    egui::FontId::default(),
                    self.theme.palette.text_secondary,
                );
            }

            ui.add_space(12.0);

            // Title
            let title = if self.state.metadata.title.is_empty() {
                "Untitled Track"
            } else {
                &self.state.metadata.title
            };
            ui.label(egui::RichText::new(title).heading().strong());

            // Artist
            let artist = if self.state.metadata.artist.is_empty() {
                "Unknown Artist"
            } else {
                &self.state.metadata.artist
            };
            ui.label(egui::RichText::new(artist).color(self.theme.palette.text_secondary));

            // Album (if set)
            if !self.state.metadata.album.is_empty() {
                ui.label(egui::RichText::new(&self.state.metadata.album)
                    .small()
                    .italics()
                    .color(self.theme.palette.text_secondary));
            }

            ui.add_space(8.0);

            // Genre and year
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&self.state.metadata.genre)
                    .small()
                    .background_color(self.theme.palette.bg_tertiary));
                ui.label(egui::RichText::new(format!("{}", self.state.metadata.year))
                    .small()
                    .color(self.theme.palette.text_secondary));
            });

            // Master file status
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            if let Some(info) = &self.state.master_info {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("✓").color(Color32::from_rgb(46, 204, 113)));
                    ui.label("Master Ready");
                });
                ui.label(egui::RichText::new(format!(
                    "{} • {}kHz • {} • {:.1} LUFS",
                    info.format,
                    info.sample_rate / 1000,
                    format_duration(info.duration_secs),
                    info.lufs
                )).small().monospace());
            } else {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("○").color(Color32::from_rgb(241, 196, 15)));
                    ui.label("No master exported");
                });
                ui.label(egui::RichText::new("Export from Master mode")
                    .small()
                    .color(self.theme.palette.text_secondary));
            }
        });
    }

    fn show_quick_stats(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(egui::RichText::new("Release Status").strong());
            ui.add_space(8.0);

            egui::Grid::new("quick_stats")
                .num_columns(2)
                .spacing([16.0, 8.0])
                .show(ui, |ui| {
                    // Metadata completion
                    let meta_pct = self.state.metadata.completion_percentage();
                    ui.label("Metadata:");
                    self.draw_progress_badge(ui, meta_pct);
                    ui.end_row();

                    // Checklist
                    let (complete, total) = self.state.checklist.completion_stats();
                    ui.label("Checklist:");
                    ui.label(format!("{}/{} items", complete, total));
                    ui.end_row();

                    // Platforms
                    let platforms = self.state.enabled_platform_count();
                    ui.label("Platforms:");
                    ui.label(format!("{} selected", platforms));
                    ui.end_row();

                    // Cover art
                    ui.label("Cover Art:");
                    if self.state.cover_art.is_valid {
                        ui.label(egui::RichText::new("✓ Valid").color(Color32::from_rgb(46, 204, 113)));
                    } else {
                        ui.label(egui::RichText::new("○ Missing").color(Color32::from_rgb(241, 196, 15)));
                    }
                    ui.end_row();
                });
        });
    }

    fn show_quick_actions(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(egui::RichText::new("Quick Actions").strong());
            ui.add_space(8.0);

            ui.horizontal_wrapped(|ui| {
                if ui.button("Edit Metadata").clicked() {
                    self.state.active_tab = DistributeTab::Metadata;
                }
                if ui.button("Add Cover Art").clicked() {
                    self.state.active_tab = DistributeTab::Artwork;
                }
                if ui.button("Select Platforms").clicked() {
                    self.state.active_tab = DistributeTab::Platforms;
                }
                if ui.button("View Checklist").clicked() {
                    self.state.active_tab = DistributeTab::Checklist;
                }
            });

            ui.add_space(8.0);

            // Main distribute button
            let is_ready = self.state.checklist.is_ready();
            let btn_text = if self.state.distribution_in_progress {
                "Distributing..."
            } else if is_ready {
                "Distribute Release"
            } else {
                "Complete Checklist First"
            };

            let btn_color = if is_ready {
                self.theme.palette.accent
            } else {
                self.theme.palette.bg_tertiary
            };

            if ui.add_enabled(!self.state.distribution_in_progress,
                egui::Button::new(egui::RichText::new(btn_text).strong())
                    .fill(btn_color)
                    .min_size(Vec2::new(200.0, 36.0))
            ).clicked() && is_ready {
                self.state.distribution_in_progress = true;
            }
        });
    }

    fn show_platform_summary(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(egui::RichText::new("Target Platforms").strong());
            ui.add_space(8.0);

            let enabled: Vec<_> = self.state.platforms.iter()
                .filter(|p| p.enabled)
                .collect();

            if enabled.is_empty() {
                ui.label(egui::RichText::new("No platforms selected")
                    .color(self.theme.palette.text_secondary)
                    .italics());
            } else {
                ui.horizontal_wrapped(|ui| {
                    for p in enabled {
                        ui.label(egui::RichText::new(format!("{} {}", p.platform.icon(), p.platform.name()))
                            .background_color(p.status.color().linear_multiply(0.3)));
                    }
                });
            }
        });
    }

    fn show_metadata_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Section selector
            ui.allocate_ui_with_layout(
                Vec2::new(120.0, ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.label(egui::RichText::new("Sections").strong());
                    ui.separator();

                    for section in [MetadataSection::Basic, MetadataSection::Album,
                                   MetadataSection::Credits, MetadataSection::Codes, MetadataSection::Legal] {
                        let is_selected = self.state.metadata_section == section;
                        if ui.selectable_label(is_selected, section_name(section)).clicked() {
                            self.state.metadata_section = section;
                        }
                    }
                },
            );

            ui.separator();

            // Metadata editor
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    match self.state.metadata_section {
                        MetadataSection::Basic => self.show_basic_metadata(ui),
                        MetadataSection::Album => self.show_album_metadata(ui),
                        MetadataSection::Credits => self.show_credits_metadata(ui),
                        MetadataSection::Codes => self.show_codes_metadata(ui),
                        MetadataSection::Legal => self.show_legal_metadata(ui),
                    }
                });
        });
    }

    fn show_basic_metadata(&mut self, ui: &mut Ui) {
        ui.heading("Basic Information");
        ui.add_space(8.0);

        egui::Grid::new("basic_metadata")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                ui.label("Title *");
                ui.add(egui::TextEdit::singleline(&mut self.state.metadata.title)
                    .hint_text("Track title")
                    .desired_width(300.0));
                ui.end_row();

                ui.label("Artist *");
                ui.add(egui::TextEdit::singleline(&mut self.state.metadata.artist)
                    .hint_text("Primary artist")
                    .desired_width(300.0));
                ui.end_row();

                ui.label("Genre *");
                egui::ComboBox::from_id_salt("genre")
                    .selected_text(&self.state.metadata.genre)
                    .width(300.0)
                    .show_ui(ui, |ui| {
                        for genre in ["Rock", "Pop", "Hip-Hop", "R&B", "Electronic", "Jazz",
                                     "Classical", "Country", "Folk", "Metal", "Indie", "Alternative"] {
                            ui.selectable_value(&mut self.state.metadata.genre, genre.to_string(), genre);
                        }
                    });
                ui.end_row();

                ui.label("Subgenre");
                ui.add(egui::TextEdit::singleline(&mut self.state.metadata.subgenre)
                    .hint_text("Optional subgenre")
                    .desired_width(300.0));
                ui.end_row();

                ui.label("Language");
                egui::ComboBox::from_id_salt("language")
                    .selected_text(language_name(&self.state.metadata.language))
                    .width(300.0)
                    .show_ui(ui, |ui| {
                        for (code, name) in [("en", "English"), ("es", "Spanish"), ("fr", "French"),
                                            ("de", "German"), ("it", "Italian"), ("pt", "Portuguese"),
                                            ("ja", "Japanese"), ("ko", "Korean"), ("zh", "Chinese")] {
                            ui.selectable_value(&mut self.state.metadata.language, code.to_string(), name);
                        }
                    });
                ui.end_row();

                ui.label("Explicit");
                ui.checkbox(&mut self.state.metadata.explicit, "Contains explicit content");
                ui.end_row();

                ui.label("BPM");
                ui.add(egui::DragValue::new(&mut self.state.metadata.bpm)
                    .range(40..=300));
                ui.end_row();

                ui.label("Key");
                egui::ComboBox::from_id_salt("key")
                    .selected_text(if self.state.metadata.key.is_empty() { "Select..." } else { &self.state.metadata.key })
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        for key in ["C", "C#/Db", "D", "D#/Eb", "E", "F", "F#/Gb", "G", "G#/Ab", "A", "A#/Bb", "B",
                                   "Cm", "C#m/Dbm", "Dm", "D#m/Ebm", "Em", "Fm", "F#m/Gbm", "Gm", "G#m/Abm", "Am", "A#m/Bbm", "Bm"] {
                            ui.selectable_value(&mut self.state.metadata.key, key.to_string(), key);
                        }
                    });
                ui.end_row();
            });

        ui.add_space(16.0);
        ui.label(egui::RichText::new("* Required fields").small().color(self.theme.palette.text_secondary));
    }

    fn show_album_metadata(&mut self, ui: &mut Ui) {
        ui.heading("Album Information");
        ui.add_space(8.0);

        egui::Grid::new("album_metadata")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                ui.label("Album");
                ui.add(egui::TextEdit::singleline(&mut self.state.metadata.album)
                    .hint_text("Album name (leave empty for single)")
                    .desired_width(300.0));
                ui.end_row();

                ui.label("Album Artist");
                ui.add(egui::TextEdit::singleline(&mut self.state.metadata.album_artist)
                    .hint_text("If different from track artist")
                    .desired_width(300.0));
                ui.end_row();

                ui.label("Track Number");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.state.metadata.track_number)
                        .range(1..=999));
                    ui.label("of");
                    ui.add(egui::DragValue::new(&mut self.state.metadata.total_tracks)
                        .range(1..=999));
                });
                ui.end_row();

                ui.label("Disc Number");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.state.metadata.disc_number)
                        .range(1..=99));
                    ui.label("of");
                    ui.add(egui::DragValue::new(&mut self.state.metadata.total_discs)
                        .range(1..=99));
                });
                ui.end_row();

                ui.label("Release Date");
                ui.add(egui::TextEdit::singleline(&mut self.state.metadata.release_date)
                    .hint_text("YYYY-MM-DD")
                    .desired_width(150.0));
                ui.end_row();

                ui.label("Year");
                ui.add(egui::DragValue::new(&mut self.state.metadata.year)
                    .range(1900..=2100));
                ui.end_row();
            });
    }

    fn show_credits_metadata(&mut self, ui: &mut Ui) {
        ui.heading("Credits");
        ui.add_space(8.0);

        egui::Grid::new("credits_metadata")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                ui.label("Composer(s)");
                ui.add(egui::TextEdit::singleline(&mut self.state.metadata.composer)
                    .hint_text("Separate multiple with commas")
                    .desired_width(300.0));
                ui.end_row();

                ui.label("Lyricist(s)");
                ui.add(egui::TextEdit::singleline(&mut self.state.metadata.lyricist)
                    .hint_text("Separate multiple with commas")
                    .desired_width(300.0));
                ui.end_row();

                ui.label("Producer(s)");
                ui.add(egui::TextEdit::singleline(&mut self.state.metadata.producer)
                    .hint_text("Separate multiple with commas")
                    .desired_width(300.0));
                ui.end_row();

                ui.label("Record Label");
                ui.add(egui::TextEdit::singleline(&mut self.state.metadata.label)
                    .hint_text("Label name or 'Independent'")
                    .desired_width(300.0));
                ui.end_row();
            });
    }

    fn show_codes_metadata(&mut self, ui: &mut Ui) {
        ui.heading("Industry Codes");
        ui.add_space(8.0);

        ui.label(egui::RichText::new("These codes are required for distribution to streaming platforms")
            .small()
            .color(self.theme.palette.text_secondary));
        ui.add_space(8.0);

        egui::Grid::new("codes_metadata")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                ui.label("ISRC *");
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.state.metadata.isrc)
                        .hint_text("XX-XXX-YY-NNNNN")
                        .desired_width(200.0));
                    if ui.small_button("Generate").on_hover_text("Generate a new ISRC (requires registration)").clicked() {
                        // Would generate ISRC
                    }
                });
                ui.end_row();

                ui.label("UPC/EAN");
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.state.metadata.upc)
                        .hint_text("12 or 13 digits")
                        .desired_width(200.0));
                    if ui.small_button("Generate").on_hover_text("Generate a new UPC").clicked() {
                        // Would generate UPC
                    }
                });
                ui.end_row();
            });

        ui.add_space(16.0);

        ui.group(|ui| {
            ui.label(egui::RichText::new("About ISRC").strong());
            ui.label("The International Standard Recording Code uniquely identifies a recording.");
            ui.label("Format: CC-XXX-YY-NNNNN (Country-Registrant-Year-Designation)");
            ui.hyperlink_to("Learn more about ISRC", "https://isrc.ifpi.org/");
        });
    }

    fn show_legal_metadata(&mut self, ui: &mut Ui) {
        ui.heading("Legal Information");
        ui.add_space(8.0);

        egui::Grid::new("legal_metadata")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                ui.label("Copyright *");
                ui.horizontal(|ui| {
                    ui.label("©");
                    ui.add(egui::DragValue::new(&mut self.state.metadata.copyright_year)
                        .range(1900..=2100));
                    ui.add(egui::TextEdit::singleline(&mut self.state.metadata.copyright)
                        .hint_text("Copyright holder")
                        .desired_width(200.0));
                });
                ui.end_row();

                ui.label("℗ Line");
                ui.add(egui::TextEdit::singleline(&mut self.state.metadata.p_line)
                    .hint_text("Sound recording copyright")
                    .desired_width(300.0));
                ui.end_row();

                ui.label("Comments");
                ui.add(egui::TextEdit::multiline(&mut self.state.metadata.comments)
                    .hint_text("Additional notes")
                    .desired_width(300.0)
                    .desired_rows(3));
                ui.end_row();
            });

        ui.add_space(16.0);

        ui.group(|ui| {
            ui.label(egui::RichText::new("Rights Confirmation").strong());
            ui.checkbox(&mut true, "I confirm I own or have the rights to distribute this recording");
            ui.checkbox(&mut true, "I confirm all credits are accurate");
        });
    }

    fn show_artwork_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Cover art preview
            ui.allocate_ui_with_layout(
                Vec2::new(350.0, ui.available_height()),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.heading("Cover Art");
                    ui.add_space(16.0);

                    // Large preview
                    let preview_size = 300.0;
                    let (rect, response) = ui.allocate_exact_size(Vec2::new(preview_size, preview_size), egui::Sense::click());

                    if self.state.cover_art.path.is_some() {
                        ui.painter().rect_filled(rect, 8.0, Color32::from_rgb(60, 60, 60));
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "Cover Art Preview",
                            egui::FontId::default(),
                            Color32::WHITE,
                        );
                    } else {
                        ui.painter().rect_filled(rect, 8.0, self.theme.palette.bg_tertiary);
                        ui.painter().rect_stroke(rect, 8.0, Stroke::new(2.0, Color32::from_gray(80)));
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "Click to Upload\nor Drag & Drop",
                            egui::FontId::default(),
                            self.theme.palette.text_secondary,
                        );
                    }

                    if response.clicked() {
                        // Would open file dialog
                    }

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.button("Upload Image").clicked() {
                            // Would open file dialog
                        }
                        if self.state.cover_art.path.is_some() {
                            if ui.button("Remove").clicked() {
                                self.state.cover_art = CoverArt::default();
                            }
                        }
                    });
                },
            );

            ui.separator();

            // Requirements and validation
            ui.vertical(|ui| {
                ui.heading("Requirements");
                ui.add_space(8.0);

                ui.group(|ui| {
                    ui.label(egui::RichText::new("Image Requirements").strong());
                    ui.add_space(4.0);

                    let requirements = [
                        ("Minimum Size", "3000 x 3000 pixels"),
                        ("Maximum Size", "10000 x 10000 pixels"),
                        ("Aspect Ratio", "Square (1:1)"),
                        ("Format", "JPEG or PNG"),
                        ("Color Space", "RGB"),
                        ("Max File Size", "20 MB"),
                    ];

                    for (label, value) in requirements {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}:", label));
                            ui.label(egui::RichText::new(value).monospace());
                        });
                    }
                });

                ui.add_space(16.0);

                // Validation status
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Validation").strong());
                    ui.add_space(4.0);

                    for msg in &self.state.cover_art.validation_messages {
                        let (icon, color) = if self.state.cover_art.is_valid {
                            ("✓", Color32::from_rgb(46, 204, 113))
                        } else if msg.contains("No cover") {
                            ("○", Color32::from_rgb(241, 196, 15))
                        } else {
                            ("✕", Color32::from_rgb(231, 76, 60))
                        };

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(icon).color(color));
                            ui.label(msg);
                        });
                    }
                });

                if let Some((w, h)) = self.state.cover_art.dimensions {
                    ui.add_space(8.0);
                    ui.label(format!("Current: {}x{} pixels", w, h));
                }
            });
        });
    }

    fn show_platforms_tab(&mut self, ui: &mut Ui) {
        ui.heading("Distribution Platforms");
        ui.label(egui::RichText::new("Select where you want to distribute your release")
            .color(self.theme.palette.text_secondary));
        ui.add_space(8.0);

        // Platform categories
        for category in [PlatformCategory::Streaming, PlatformCategory::Video,
                        PlatformCategory::Independent, PlatformCategory::Social] {
            ui.group(|ui| {
                ui.label(egui::RichText::new(category.name()).strong());
                ui.add_space(4.0);

                let platforms: Vec<_> = self.state.platforms.iter_mut()
                    .filter(|p| p.platform.category() == category)
                    .collect();

                for platform in platforms {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut platform.enabled, "");
                        ui.label(format!("{} {}", platform.platform.icon(), platform.platform.name()));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Status badge
                            if platform.enabled {
                                let badge = egui::RichText::new(platform.status.label())
                                    .small()
                                    .color(platform.status.color());
                                ui.label(badge);
                            }
                        });
                    });

                    if platform.enabled {
                        ui.horizontal(|ui| {
                            ui.add_space(24.0);
                            ui.label(egui::RichText::new(platform.platform.requirements())
                                .small()
                                .color(self.theme.palette.text_secondary));
                        });
                    }
                }
            });
            ui.add_space(8.0);
        }

        // Summary
        ui.separator();
        let enabled_count = self.state.enabled_platform_count();
        ui.label(format!("{} platform{} selected", enabled_count, if enabled_count == 1 { "" } else { "s" }));
    }

    fn show_checklist_tab(&mut self, ui: &mut Ui) {
        ui.heading("Pre-Release Checklist");

        let (complete, total) = self.state.checklist.completion_stats();
        ui.label(format!("{} of {} required items complete", complete, total));
        ui.add_space(8.0);

        // Progress bar
        let progress = if total > 0 { complete as f32 / total as f32 } else { 0.0 };
        ui.add(egui::ProgressBar::new(progress).show_percentage());
        ui.add_space(16.0);

        // Checklist by category
        for category in [ChecklistCategory::Metadata, ChecklistCategory::Audio,
                        ChecklistCategory::Artwork, ChecklistCategory::Legal] {
            let items: Vec<_> = self.state.checklist.items.iter()
                .filter(|i| i.category == category)
                .collect();

            if items.is_empty() { continue; }

            ui.group(|ui| {
                ui.label(egui::RichText::new(category.name()).strong());
                ui.add_space(4.0);

                for item in items {
                    ui.horizontal(|ui| {
                        let (icon, color) = if item.is_complete {
                            ("✓", Color32::from_rgb(46, 204, 113))
                        } else if item.is_required {
                            ("○", Color32::from_rgb(231, 76, 60))
                        } else {
                            ("○", Color32::from_rgb(241, 196, 15))
                        };

                        ui.label(egui::RichText::new(icon).color(color));
                        ui.label(item.label);

                        if item.is_required {
                            ui.label(egui::RichText::new("*").color(Color32::from_rgb(231, 76, 60)));
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new(item.description)
                            .small()
                            .color(self.theme.palette.text_secondary));
                    });
                }
            });
            ui.add_space(8.0);
        }
    }

    fn show_promo_tab(&mut self, ui: &mut Ui) {
        ui.heading("Promotional Materials");
        ui.label(egui::RichText::new("Generate marketing assets for your release")
            .color(self.theme.palette.text_secondary));
        ui.add_space(16.0);

        // Promo materials grid
        egui::Grid::new("promo_grid")
            .num_columns(2)
            .spacing([16.0, 16.0])
            .show(ui, |ui| {
                for material in &mut self.state.promo_materials {
                    ui.group(|ui| {
                        ui.set_min_width(250.0);

                        ui.label(egui::RichText::new(material.material_type.name()).strong());
                        ui.label(egui::RichText::new(material.material_type.description())
                            .small()
                            .color(self.theme.palette.text_secondary));

                        ui.add_space(8.0);

                        if material.generated {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("✓ Generated")
                                    .color(Color32::from_rgb(46, 204, 113)));
                                if ui.small_button("View").clicked() {
                                    // Would open preview
                                }
                                if ui.small_button("Export").clicked() {
                                    // Would export
                                }
                            });
                        } else {
                            if ui.button("Generate").clicked() {
                                material.generated = true; // Simulated
                            }
                        }
                    });
                }
            });

        ui.add_space(16.0);

        // Bulk actions
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Generate All").clicked() {
                for material in &mut self.state.promo_materials {
                    material.generated = true;
                }
            }
            if ui.button("Export All").clicked() {
                // Would export all
            }
        });
    }

    fn show_status_bar(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Completion status
            let (complete, total) = self.state.checklist.completion_stats();
            ui.label(egui::RichText::new(format!("Checklist: {}/{}", complete, total))
                .small());

            ui.separator();

            // Platform count
            let platforms = self.state.enabled_platform_count();
            ui.label(egui::RichText::new(format!("{} platforms", platforms))
                .small());

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("1-6: Tabs")
                    .small()
                    .color(self.theme.palette.text_secondary));
            });
        });
    }

    fn draw_progress_badge(&self, ui: &mut Ui, percentage: u8) {
        let color = if percentage >= 100 {
            Color32::from_rgb(46, 204, 113)
        } else if percentage >= 70 {
            Color32::from_rgb(241, 196, 15)
        } else {
            Color32::from_rgb(231, 76, 60)
        };

        ui.label(egui::RichText::new(format!("{}%", percentage))
            .color(color)
            .monospace());
    }

    fn handle_keyboard(&mut self, ui: &mut Ui) {
        ui.input(|i| {
            // Number keys for tabs
            if i.key_pressed(egui::Key::Num1) {
                self.state.active_tab = DistributeTab::Overview;
            }
            if i.key_pressed(egui::Key::Num2) {
                self.state.active_tab = DistributeTab::Metadata;
            }
            if i.key_pressed(egui::Key::Num3) {
                self.state.active_tab = DistributeTab::Artwork;
            }
            if i.key_pressed(egui::Key::Num4) {
                self.state.active_tab = DistributeTab::Platforms;
            }
            if i.key_pressed(egui::Key::Num5) {
                self.state.active_tab = DistributeTab::Checklist;
            }
            if i.key_pressed(egui::Key::Num6) {
                self.state.active_tab = DistributeTab::Promo;
            }
        });
    }
}

// Helper functions
fn format_duration(secs: f32) -> String {
    let mins = (secs / 60.0) as u32;
    let secs = secs % 60.0;
    format!("{}:{:02}", mins, secs as u32)
}

fn section_name(section: MetadataSection) -> &'static str {
    match section {
        MetadataSection::Basic => "Basic",
        MetadataSection::Album => "Album",
        MetadataSection::Credits => "Credits",
        MetadataSection::Codes => "Codes",
        MetadataSection::Legal => "Legal",
    }
}

fn language_name(code: &str) -> &'static str {
    match code {
        "en" => "English",
        "es" => "Spanish",
        "fr" => "French",
        "de" => "German",
        "it" => "Italian",
        "pt" => "Portuguese",
        "ja" => "Japanese",
        "ko" => "Korean",
        "zh" => "Chinese",
        _ => "Unknown",
    }
}
