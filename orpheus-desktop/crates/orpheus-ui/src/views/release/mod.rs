//! Release mode view - artwork, visualizers, and promotional materials

use egui::{Color32, Ui, ScrollArea, Vec2, Stroke, Rect, Pos2};
use crate::theme::Theme;
use std::path::PathBuf;

// ============================================================================
// Artwork Studio - Album Art Generation and Design
// ============================================================================

/// Style presets for artwork generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtworkStyle {
    /// Minimalist design with typography focus
    Minimalist,
    /// Abstract generative art from audio
    Generative,
    /// Geometric shapes and patterns
    Geometric,
    /// Photo-based with effects
    Photographic,
    /// Illustrated/drawn style
    Illustrated,
    /// Collage/mixed media
    Collage,
    /// Vintage/retro aesthetic
    Vintage,
    /// Modern/contemporary
    Modern,
    /// Dark/gothic
    Dark,
    /// Bright/vibrant
    Vibrant,
    /// Custom AI-generated
    AiGenerated,
}

impl ArtworkStyle {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Minimalist => "Minimalist",
            Self::Generative => "Generative",
            Self::Geometric => "Geometric",
            Self::Photographic => "Photographic",
            Self::Illustrated => "Illustrated",
            Self::Collage => "Collage",
            Self::Vintage => "Vintage",
            Self::Modern => "Modern",
            Self::Dark => "Dark",
            Self::Vibrant => "Vibrant",
            Self::AiGenerated => "AI Generated",
        }
    }

    pub fn all() -> &'static [ArtworkStyle] {
        &[
            Self::Minimalist,
            Self::Generative,
            Self::Geometric,
            Self::Photographic,
            Self::Illustrated,
            Self::Collage,
            Self::Vintage,
            Self::Modern,
            Self::Dark,
            Self::Vibrant,
            Self::AiGenerated,
        ]
    }
}

/// Color palette for artwork
#[derive(Clone)]
pub struct ColorPalette {
    /// Palette name
    pub name: String,
    /// Primary colors (up to 5)
    pub colors: Vec<Color32>,
    /// Whether extracted from audio mood
    pub from_audio: bool,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            colors: vec![
                Color32::from_rgb(41, 128, 185),   // Blue
                Color32::from_rgb(142, 68, 173),   // Purple
                Color32::from_rgb(39, 174, 96),    // Green
                Color32::from_rgb(241, 196, 15),   // Yellow
                Color32::from_rgb(231, 76, 60),    // Red
            ],
            from_audio: false,
        }
    }
}

/// Typography settings for artwork
#[derive(Clone)]
pub struct TypographySettings {
    /// Font family name
    pub font_family: String,
    /// Title font size
    pub title_size: f32,
    /// Artist name font size
    pub artist_size: f32,
    /// Text color
    pub text_color: Color32,
    /// Text alignment
    pub alignment: TextAlignment,
    /// Text position
    pub position: TextPosition,
    /// Add text shadow
    pub shadow: bool,
    /// Add text outline
    pub outline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextPosition {
    Top,
    Center,
    Bottom,
    Custom,
}

impl Default for TypographySettings {
    fn default() -> Self {
        Self {
            font_family: "Inter".to_string(),
            title_size: 72.0,
            artist_size: 36.0,
            text_color: Color32::WHITE,
            alignment: TextAlignment::Center,
            position: TextPosition::Bottom,
            shadow: true,
            outline: false,
        }
    }
}

/// Artwork project state
#[derive(Clone)]
pub struct ArtworkProject {
    /// Project name
    pub name: String,
    /// Canvas dimensions (always square for album art)
    pub dimensions: u32,
    /// Selected style preset
    pub style: ArtworkStyle,
    /// Color palette
    pub palette: ColorPalette,
    /// Typography settings
    pub typography: TypographySettings,
    /// Background image path
    pub background_image: Option<PathBuf>,
    /// Overlay elements
    pub overlays: Vec<ArtworkOverlay>,
    /// AI prompt for generation
    pub ai_prompt: String,
    /// Export path
    pub export_path: Option<PathBuf>,
}

#[derive(Clone)]
pub struct ArtworkOverlay {
    pub overlay_type: OverlayType,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub position: (f32, f32),
    pub scale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayType {
    Gradient,
    Noise,
    Texture,
    Geometric,
    Waveform,
    Spectrum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    SoftLight,
    HardLight,
}

impl Default for ArtworkProject {
    fn default() -> Self {
        Self {
            name: "Untitled Artwork".to_string(),
            dimensions: 3000,
            style: ArtworkStyle::Modern,
            palette: ColorPalette::default(),
            typography: TypographySettings::default(),
            background_image: None,
            overlays: Vec::new(),
            ai_prompt: String::new(),
            export_path: None,
        }
    }
}

// ============================================================================
// Visualizer Generator - Audio Visualizers and Waveform Art
// ============================================================================

/// Visualizer types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualizerType {
    /// Classic waveform display
    Waveform,
    /// Frequency spectrum bars
    SpectrumBars,
    /// Circular spectrum
    CircularSpectrum,
    /// Particle system reactive to audio
    Particles,
    /// 3D terrain from audio
    Terrain,
    /// Geometric patterns
    Geometric,
    /// Liquid/fluid simulation
    Fluid,
    /// Line art reactive
    LineArt,
    /// Typography animation
    KineticType,
    /// Custom shader
    CustomShader,
}

impl VisualizerType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Waveform => "Waveform",
            Self::SpectrumBars => "Spectrum Bars",
            Self::CircularSpectrum => "Circular Spectrum",
            Self::Particles => "Particles",
            Self::Terrain => "3D Terrain",
            Self::Geometric => "Geometric",
            Self::Fluid => "Fluid",
            Self::LineArt => "Line Art",
            Self::KineticType => "Kinetic Typography",
            Self::CustomShader => "Custom Shader",
        }
    }

    pub fn all() -> &'static [VisualizerType] {
        &[
            Self::Waveform,
            Self::SpectrumBars,
            Self::CircularSpectrum,
            Self::Particles,
            Self::Terrain,
            Self::Geometric,
            Self::Fluid,
            Self::LineArt,
            Self::KineticType,
            Self::CustomShader,
        ]
    }
}

/// Video export settings
#[derive(Clone)]
pub struct VideoExportSettings {
    /// Output resolution
    pub resolution: VideoResolution,
    /// Frame rate
    pub fps: u32,
    /// Video codec
    pub codec: VideoCodec,
    /// Quality (0-100)
    pub quality: u8,
    /// Include audio
    pub include_audio: bool,
    /// Loop count (0 = infinite for GIF)
    pub loop_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoResolution {
    HD720,      // 1280x720
    HD1080,     // 1920x1080
    UHD4K,      // 3840x2160
    Square1080, // 1080x1080 (Instagram)
    Portrait,   // 1080x1920 (Stories/TikTok)
    Custom,
}

impl VideoResolution {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Self::HD720 => (1280, 720),
            Self::HD1080 => (1920, 1080),
            Self::UHD4K => (3840, 2160),
            Self::Square1080 => (1080, 1080),
            Self::Portrait => (1080, 1920),
            Self::Custom => (1920, 1080),
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::HD720 => "720p (1280×720)",
            Self::HD1080 => "1080p (1920×1080)",
            Self::UHD4K => "4K (3840×2160)",
            Self::Square1080 => "Square (1080×1080)",
            Self::Portrait => "Portrait (1080×1920)",
            Self::Custom => "Custom",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodec {
    H264,
    H265,
    VP9,
    ProRes,
    GIF,
}

impl Default for VideoExportSettings {
    fn default() -> Self {
        Self {
            resolution: VideoResolution::HD1080,
            fps: 30,
            codec: VideoCodec::H264,
            quality: 85,
            include_audio: true,
            loop_count: 1,
        }
    }
}

/// Visualizer project state
#[derive(Clone)]
pub struct VisualizerProject {
    /// Project name
    pub name: String,
    /// Visualizer type
    pub viz_type: VisualizerType,
    /// Color scheme
    pub colors: ColorPalette,
    /// Background color
    pub background: Color32,
    /// Sensitivity/reactivity
    pub sensitivity: f32,
    /// Smoothing factor
    pub smoothing: f32,
    /// Export settings
    pub export: VideoExportSettings,
    /// Custom parameters
    pub params: VisualizerParams,
}

#[derive(Clone, Default)]
pub struct VisualizerParams {
    /// Bar count for spectrum
    pub bar_count: u32,
    /// Particle count
    pub particle_count: u32,
    /// Line thickness
    pub line_thickness: f32,
    /// Glow intensity
    pub glow: f32,
    /// Mirror mode
    pub mirror: bool,
    /// Rotation speed
    pub rotation: f32,
    /// Zoom level
    pub zoom: f32,
}

impl Default for VisualizerProject {
    fn default() -> Self {
        Self {
            name: "Untitled Visualizer".to_string(),
            viz_type: VisualizerType::SpectrumBars,
            colors: ColorPalette::default(),
            background: Color32::BLACK,
            sensitivity: 1.0,
            smoothing: 0.3,
            export: VideoExportSettings::default(),
            params: VisualizerParams {
                bar_count: 64,
                particle_count: 1000,
                line_thickness: 2.0,
                glow: 0.5,
                mirror: true,
                rotation: 0.0,
                zoom: 1.0,
            },
        }
    }
}

// ============================================================================
// Promotional Materials - Social Media and Press Kit
// ============================================================================

/// Social media platform targets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocialPlatform {
    Instagram,
    InstagramStory,
    Facebook,
    Twitter,
    YouTube,
    YouTubeThumbnail,
    TikTok,
    Spotify,
    AppleMusic,
    SoundCloud,
    Bandcamp,
    Website,
}

impl SocialPlatform {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Instagram => "Instagram Post",
            Self::InstagramStory => "Instagram Story",
            Self::Facebook => "Facebook",
            Self::Twitter => "Twitter/X",
            Self::YouTube => "YouTube Banner",
            Self::YouTubeThumbnail => "YouTube Thumbnail",
            Self::TikTok => "TikTok",
            Self::Spotify => "Spotify Canvas",
            Self::AppleMusic => "Apple Music",
            Self::SoundCloud => "SoundCloud",
            Self::Bandcamp => "Bandcamp",
            Self::Website => "Website Banner",
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Self::Instagram => (1080, 1080),
            Self::InstagramStory => (1080, 1920),
            Self::Facebook => (1200, 630),
            Self::Twitter => (1200, 675),
            Self::YouTube => (2560, 1440),
            Self::YouTubeThumbnail => (1280, 720),
            Self::TikTok => (1080, 1920),
            Self::Spotify => (720, 720),
            Self::AppleMusic => (2400, 1350),
            Self::SoundCloud => (2480, 520),
            Self::Bandcamp => (700, 700),
            Self::Website => (1920, 400),
        }
    }

    pub fn all() -> &'static [SocialPlatform] {
        &[
            Self::Instagram,
            Self::InstagramStory,
            Self::Facebook,
            Self::Twitter,
            Self::YouTube,
            Self::YouTubeThumbnail,
            Self::TikTok,
            Self::Spotify,
            Self::AppleMusic,
            Self::SoundCloud,
            Self::Bandcamp,
            Self::Website,
        ]
    }
}

/// Promo asset state
#[derive(Clone)]
pub struct PromoAsset {
    /// Platform target
    pub platform: SocialPlatform,
    /// Generated/ready
    pub generated: bool,
    /// Custom text overlay
    pub text_overlay: Option<String>,
    /// Release date text
    pub show_date: bool,
    /// Streaming links text
    pub show_links: bool,
    /// Export path
    pub export_path: Option<PathBuf>,
}

/// Press kit / EPK content
#[derive(Clone, Default)]
pub struct PressKit {
    /// Artist bio (short)
    pub bio_short: String,
    /// Artist bio (long)
    pub bio_long: String,
    /// Press release text
    pub press_release: String,
    /// Genre tags
    pub genres: Vec<String>,
    /// Influences/similar artists
    pub influences: Vec<String>,
    /// Press photos
    pub photos: Vec<PathBuf>,
    /// Logo files
    pub logos: Vec<PathBuf>,
    /// Social links
    pub social_links: SocialLinks,
    /// Contact info
    pub contact_email: String,
    /// Management contact
    pub management: String,
    /// Booking contact
    pub booking: String,
}

#[derive(Clone, Default)]
pub struct SocialLinks {
    pub website: String,
    pub instagram: String,
    pub twitter: String,
    pub facebook: String,
    pub youtube: String,
    pub tiktok: String,
    pub spotify: String,
    pub apple_music: String,
    pub soundcloud: String,
    pub bandcamp: String,
}

// ============================================================================
// Merchandise Mockups
// ============================================================================

/// Merchandise product types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MerchType {
    TShirt,
    Hoodie,
    Poster,
    Vinyl,
    CD,
    Cassette,
    Sticker,
    ToteBag,
    Mug,
    PhoneCase,
}

impl MerchType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::TShirt => "T-Shirt",
            Self::Hoodie => "Hoodie",
            Self::Poster => "Poster",
            Self::Vinyl => "Vinyl Record",
            Self::CD => "CD",
            Self::Cassette => "Cassette",
            Self::Sticker => "Sticker",
            Self::ToteBag => "Tote Bag",
            Self::Mug => "Mug",
            Self::PhoneCase => "Phone Case",
        }
    }

    pub fn all() -> &'static [MerchType] {
        &[
            Self::TShirt,
            Self::Hoodie,
            Self::Poster,
            Self::Vinyl,
            Self::CD,
            Self::Cassette,
            Self::Sticker,
            Self::ToteBag,
            Self::Mug,
            Self::PhoneCase,
        ]
    }
}

/// Merchandise mockup state
#[derive(Clone)]
pub struct MerchMockup {
    /// Product type
    pub product: MerchType,
    /// Color variant
    pub color: Color32,
    /// Artwork placement
    pub placement: MerchPlacement,
    /// Scale of artwork
    pub scale: f32,
    /// Generated preview path
    pub preview_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MerchPlacement {
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    FullBleed,
}

// ============================================================================
// Release View State and Tabs
// ============================================================================

/// Main tabs for Release view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseTab {
    /// Overview and workflow
    Overview,
    /// Artwork studio
    Artwork,
    /// Visualizer generator
    Visualizer,
    /// Social media assets
    SocialMedia,
    /// Press kit / EPK
    PressKit,
    /// Merchandise mockups
    Merch,
}

impl ReleaseTab {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Artwork => "Artwork",
            Self::Visualizer => "Visualizer",
            Self::SocialMedia => "Social Media",
            Self::PressKit => "Press Kit",
            Self::Merch => "Merch",
        }
    }

    pub fn all() -> &'static [ReleaseTab] {
        &[
            Self::Overview,
            Self::Artwork,
            Self::Visualizer,
            Self::SocialMedia,
            Self::PressKit,
            Self::Merch,
        ]
    }
}

/// Artwork editor sub-tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtworkTab {
    Canvas,
    Style,
    Colors,
    Typography,
    Overlays,
    AiGenerate,
}

/// Visualizer editor sub-tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualizerTab {
    Preview,
    Type,
    Colors,
    Parameters,
    Export,
}

/// Complete Release view state
pub struct ReleaseViewState {
    /// Current main tab
    pub current_tab: ReleaseTab,
    /// Artwork sub-tab
    pub artwork_tab: ArtworkTab,
    /// Visualizer sub-tab
    pub visualizer_tab: VisualizerTab,
    /// Current artwork project
    pub artwork: ArtworkProject,
    /// Current visualizer project
    pub visualizer: VisualizerProject,
    /// Generated promo assets
    pub promo_assets: Vec<PromoAsset>,
    /// Press kit content
    pub press_kit: PressKit,
    /// Merch mockups
    pub merch_mockups: Vec<MerchMockup>,
    /// Release title (from project)
    pub release_title: String,
    /// Artist name (from project)
    pub artist_name: String,
    /// Selected promo platforms
    pub selected_platforms: Vec<SocialPlatform>,
    /// Preview animation time
    pub preview_time: f32,
    /// Is generating
    pub is_generating: bool,
    /// Generation progress
    pub generation_progress: f32,
    /// Status message
    pub status_message: String,
}

impl Default for ReleaseViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl ReleaseViewState {
    pub fn new() -> Self {
        Self {
            current_tab: ReleaseTab::Overview,
            artwork_tab: ArtworkTab::Canvas,
            visualizer_tab: VisualizerTab::Preview,
            artwork: ArtworkProject::default(),
            visualizer: VisualizerProject::default(),
            promo_assets: Vec::new(),
            press_kit: PressKit::default(),
            merch_mockups: Vec::new(),
            release_title: "Untitled Release".to_string(),
            artist_name: "Artist".to_string(),
            selected_platforms: vec![
                SocialPlatform::Instagram,
                SocialPlatform::InstagramStory,
                SocialPlatform::Twitter,
                SocialPlatform::YouTube,
            ],
            preview_time: 0.0,
            is_generating: false,
            generation_progress: 0.0,
            status_message: String::new(),
        }
    }

    /// Initialize promo assets for selected platforms
    pub fn init_promo_assets(&mut self) {
        self.promo_assets = self.selected_platforms.iter().map(|&platform| {
            PromoAsset {
                platform,
                generated: false,
                text_overlay: None,
                show_date: true,
                show_links: true,
                export_path: None,
            }
        }).collect();
    }

    /// Add default merch mockups
    pub fn init_merch_mockups(&mut self) {
        self.merch_mockups = vec![
            MerchMockup {
                product: MerchType::TShirt,
                color: Color32::BLACK,
                placement: MerchPlacement::Center,
                scale: 0.6,
                preview_path: None,
            },
            MerchMockup {
                product: MerchType::Vinyl,
                color: Color32::BLACK,
                placement: MerchPlacement::FullBleed,
                scale: 1.0,
                preview_path: None,
            },
            MerchMockup {
                product: MerchType::Poster,
                color: Color32::WHITE,
                placement: MerchPlacement::FullBleed,
                scale: 1.0,
                preview_path: None,
            },
        ];
    }
}

// ============================================================================
// Release View Component
// ============================================================================

/// Release view component
pub struct ReleaseView<'a> {
    state: &'a mut ReleaseViewState,
    theme: &'a Theme,
}

impl<'a> ReleaseView<'a> {
    pub fn new(state: &'a mut ReleaseViewState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        // Handle keyboard shortcuts
        self.handle_input(ui);

        // Main layout
        ui.vertical(|ui| {
            // Tab bar
            self.show_tab_bar(ui);
            ui.add_space(8.0);

            // Content area
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    match self.state.current_tab {
                        ReleaseTab::Overview => self.show_overview(ui),
                        ReleaseTab::Artwork => self.show_artwork_studio(ui),
                        ReleaseTab::Visualizer => self.show_visualizer(ui),
                        ReleaseTab::SocialMedia => self.show_social_media(ui),
                        ReleaseTab::PressKit => self.show_press_kit(ui),
                        ReleaseTab::Merch => self.show_merch(ui),
                    }
                });

            // Status bar
            self.show_status_bar(ui);
        });
    }

    fn handle_input(&mut self, ui: &mut Ui) {
        ui.input(|i| {
            // Tab switching with number keys
            if i.key_pressed(egui::Key::Num1) {
                self.state.current_tab = ReleaseTab::Overview;
            } else if i.key_pressed(egui::Key::Num2) {
                self.state.current_tab = ReleaseTab::Artwork;
            } else if i.key_pressed(egui::Key::Num3) {
                self.state.current_tab = ReleaseTab::Visualizer;
            } else if i.key_pressed(egui::Key::Num4) {
                self.state.current_tab = ReleaseTab::SocialMedia;
            } else if i.key_pressed(egui::Key::Num5) {
                self.state.current_tab = ReleaseTab::PressKit;
            } else if i.key_pressed(egui::Key::Num6) {
                self.state.current_tab = ReleaseTab::Merch;
            }

            // G for generate
            if i.key_pressed(egui::Key::G) && !self.state.is_generating {
                self.state.status_message = "Generation started...".to_string();
            }

            // E for export
            if i.key_pressed(egui::Key::E) {
                self.state.status_message = "Export dialog...".to_string();
            }
        });
    }

    fn show_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for (idx, tab) in ReleaseTab::all().iter().enumerate() {
                let selected = self.state.current_tab == *tab;
                let text = format!("{} {}", idx + 1, tab.display_name());

                let btn = egui::Button::new(
                    egui::RichText::new(&text)
                        .color(if selected { self.theme.palette.accent } else { self.theme.text_primary() })
                )
                .fill(if selected {
                    self.theme.palette.accent.gamma_multiply(0.2)
                } else {
                    Color32::TRANSPARENT
                });

                if ui.add(btn).clicked() {
                    self.state.current_tab = *tab;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.state.is_generating {
                    ui.spinner();
                    ui.label(format!("{:.0}%", self.state.generation_progress * 100.0));
                }
            });
        });
    }

    fn show_status_bar(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&self.state.status_message)
                    .color(self.theme.text_secondary())
                    .small()
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("1-6: Tabs | G: Generate | E: Export")
                        .color(self.theme.text_secondary())
                        .small()
                );
            });
        });
    }

    // ========================================================================
    // Overview Tab
    // ========================================================================

    fn show_overview(&mut self, ui: &mut Ui) {
        ui.heading("Release Preparation");
        ui.add_space(8.0);

        ui.label("Prepare your release with professional artwork, visualizers, and promotional materials.");
        ui.add_space(16.0);

        // Release info
        ui.group(|ui| {
            ui.heading("Release Information");
            ui.add_space(8.0);

            egui::Grid::new("release_info_grid")
                .num_columns(2)
                .spacing([20.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Title:");
                    ui.text_edit_singleline(&mut self.state.release_title);
                    ui.end_row();

                    ui.label("Artist:");
                    ui.text_edit_singleline(&mut self.state.artist_name);
                    ui.end_row();
                });
        });

        ui.add_space(16.0);

        // Workflow checklist
        ui.group(|ui| {
            ui.heading("Workflow Checklist");
            ui.add_space(8.0);

            let steps = [
                ("1. Create Artwork", "Design album/single cover art", self.state.artwork.export_path.is_some()),
                ("2. Generate Visualizer", "Create audio visualizer video", false),
                ("3. Social Media Assets", "Generate platform-specific graphics", !self.state.promo_assets.is_empty()),
                ("4. Press Kit", "Prepare EPK and press materials", !self.state.press_kit.bio_short.is_empty()),
                ("5. Merch Mockups", "Preview merchandise designs", !self.state.merch_mockups.is_empty()),
            ];

            for (title, desc, done) in steps {
                ui.horizontal(|ui| {
                    let icon = if done { "✓" } else { "○" };
                    let color = if done { self.theme.palette.success } else { self.theme.text_secondary() };
                    ui.label(egui::RichText::new(icon).color(color));
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(title).strong());
                        ui.label(egui::RichText::new(desc).color(self.theme.text_secondary()).small());
                    });
                });
                ui.add_space(4.0);
            }
        });

        ui.add_space(16.0);

        // Quick actions
        ui.group(|ui| {
            ui.heading("Quick Actions");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("🎨 Open Artwork Studio").clicked() {
                    self.state.current_tab = ReleaseTab::Artwork;
                }
                if ui.button("🎬 Create Visualizer").clicked() {
                    self.state.current_tab = ReleaseTab::Visualizer;
                }
                if ui.button("📱 Generate Social Assets").clicked() {
                    self.state.current_tab = ReleaseTab::SocialMedia;
                    self.state.init_promo_assets();
                }
            });

            ui.horizontal(|ui| {
                if ui.button("📰 Edit Press Kit").clicked() {
                    self.state.current_tab = ReleaseTab::PressKit;
                }
                if ui.button("👕 Preview Merch").clicked() {
                    self.state.current_tab = ReleaseTab::Merch;
                    if self.state.merch_mockups.is_empty() {
                        self.state.init_merch_mockups();
                    }
                }
            });
        });
    }

    // ========================================================================
    // Artwork Studio Tab
    // ========================================================================

    fn show_artwork_studio(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Left: Canvas/Preview
            ui.vertical(|ui| {
                ui.set_min_width(500.0);
                self.show_artwork_canvas(ui);
            });

            ui.separator();

            // Right: Controls
            ScrollArea::vertical()
                .id_salt("artwork_controls")
                .show(ui, |ui| {
                    ui.set_min_width(350.0);
                    self.show_artwork_controls(ui);
                });
        });
    }

    fn show_artwork_canvas(&mut self, ui: &mut Ui) {
        ui.heading("Canvas Preview");
        ui.add_space(8.0);

        // Canvas area (square preview)
        let available = ui.available_size();
        let canvas_size = available.x.min(available.y - 100.0).min(500.0);

        let (rect, _response) = ui.allocate_exact_size(
            Vec2::splat(canvas_size),
            egui::Sense::click_and_drag()
        );

        // Draw canvas background
        let painter = ui.painter_at(rect);

        // Background with palette primary color
        let bg_color = self.state.artwork.palette.colors.first()
            .copied()
            .unwrap_or(Color32::from_rgb(30, 30, 40));
        painter.rect_filled(rect, 0.0, bg_color);

        // Draw simulated artwork elements
        self.draw_artwork_preview(&painter, rect);

        // Canvas info
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(format!(
                "{}×{} px",
                self.state.artwork.dimensions,
                self.state.artwork.dimensions
            ));
            ui.label("|");
            ui.label(self.state.artwork.style.display_name());
        });

        // Export button
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("📥 Export PNG").clicked() {
                self.state.status_message = "Exporting artwork...".to_string();
            }
            if ui.button("📥 Export JPEG").clicked() {
                self.state.status_message = "Exporting artwork...".to_string();
            }
        });
    }

    fn draw_artwork_preview(&self, painter: &egui::Painter, rect: Rect) {
        let center = rect.center();
        let size = rect.width();

        // Draw style-specific elements
        match self.state.artwork.style {
            ArtworkStyle::Minimalist => {
                // Simple geometric shapes
                let circle_radius = size * 0.15;
                if let Some(color) = self.state.artwork.palette.colors.get(1) {
                    painter.circle_filled(center, circle_radius, *color);
                }
            }
            ArtworkStyle::Geometric => {
                // Multiple geometric shapes
                for (i, color) in self.state.artwork.palette.colors.iter().enumerate() {
                    let offset = i as f32 * 20.0;
                    let shape_rect = Rect::from_center_size(
                        center + Vec2::new(offset - 40.0, offset - 40.0),
                        Vec2::splat(size * 0.2)
                    );
                    painter.rect_filled(shape_rect, 4.0, color.gamma_multiply(0.8));
                }
            }
            ArtworkStyle::Generative => {
                // Waveform-style lines
                let line_count = 20;
                for i in 0..line_count {
                    let y = rect.top() + (i as f32 / line_count as f32) * rect.height();
                    let amplitude = (i as f32 * 0.5).sin() * 20.0;
                    let color_idx = i % self.state.artwork.palette.colors.len();
                    let color = self.state.artwork.palette.colors.get(color_idx)
                        .copied()
                        .unwrap_or(Color32::WHITE);

                    painter.line_segment(
                        [
                            Pos2::new(rect.left() + 20.0, y + amplitude),
                            Pos2::new(rect.right() - 20.0, y - amplitude),
                        ],
                        Stroke::new(2.0, color.gamma_multiply(0.6))
                    );
                }
            }
            _ => {
                // Default: gradient-like effect with palette colors
                for (i, color) in self.state.artwork.palette.colors.iter().enumerate() {
                    let y_offset = (i as f32 / self.state.artwork.palette.colors.len() as f32) * size;
                    let band_rect = Rect::from_min_size(
                        Pos2::new(rect.left(), rect.top() + y_offset),
                        Vec2::new(size, size / self.state.artwork.palette.colors.len() as f32)
                    );
                    painter.rect_filled(band_rect, 0.0, color.gamma_multiply(0.4));
                }
            }
        }

        // Draw text overlay
        let title_pos = match self.state.artwork.typography.position {
            TextPosition::Top => Pos2::new(center.x, rect.top() + 60.0),
            TextPosition::Center => center,
            TextPosition::Bottom => Pos2::new(center.x, rect.bottom() - 80.0),
            TextPosition::Custom => center,
        };

        // Title
        painter.text(
            title_pos,
            egui::Align2::CENTER_CENTER,
            &self.state.release_title,
            egui::FontId::proportional(24.0),
            self.state.artwork.typography.text_color,
        );

        // Artist name
        painter.text(
            title_pos + Vec2::new(0.0, 30.0),
            egui::Align2::CENTER_CENTER,
            &self.state.artist_name,
            egui::FontId::proportional(16.0),
            self.state.artwork.typography.text_color.gamma_multiply(0.8),
        );
    }

    fn show_artwork_controls(&mut self, ui: &mut Ui) {
        // Sub-tab selector
        ui.horizontal(|ui| {
            for tab in [ArtworkTab::Canvas, ArtworkTab::Style, ArtworkTab::Colors, ArtworkTab::Typography, ArtworkTab::AiGenerate] {
                let name = match tab {
                    ArtworkTab::Canvas => "Canvas",
                    ArtworkTab::Style => "Style",
                    ArtworkTab::Colors => "Colors",
                    ArtworkTab::Typography => "Type",
                    ArtworkTab::Overlays => "Overlays",
                    ArtworkTab::AiGenerate => "AI",
                };
                if ui.selectable_label(self.state.artwork_tab == tab, name).clicked() {
                    self.state.artwork_tab = tab;
                }
            }
        });

        ui.separator();

        match self.state.artwork_tab {
            ArtworkTab::Canvas => self.show_canvas_settings(ui),
            ArtworkTab::Style => self.show_style_settings(ui),
            ArtworkTab::Colors => self.show_color_settings(ui),
            ArtworkTab::Typography => self.show_typography_settings(ui),
            ArtworkTab::Overlays => self.show_overlay_settings(ui),
            ArtworkTab::AiGenerate => self.show_ai_generate(ui),
        }
    }

    fn show_canvas_settings(&mut self, ui: &mut Ui) {
        ui.heading("Canvas Settings");
        ui.add_space(8.0);

        ui.label("Dimensions:");
        ui.horizontal(|ui| {
            for size in [1400, 3000, 4000] {
                if ui.selectable_label(
                    self.state.artwork.dimensions == size,
                    format!("{}px", size)
                ).clicked() {
                    self.state.artwork.dimensions = size;
                }
            }
        });

        ui.add_space(12.0);

        ui.label("Background Image:");
        ui.horizontal(|ui| {
            if ui.button("📂 Import Image").clicked() {
                self.state.status_message = "Select background image...".to_string();
            }
            if self.state.artwork.background_image.is_some() {
                if ui.button("✕ Remove").clicked() {
                    self.state.artwork.background_image = None;
                }
            }
        });

        if let Some(path) = &self.state.artwork.background_image {
            ui.label(
                egui::RichText::new(path.file_name().unwrap_or_default().to_string_lossy())
                    .color(self.theme.text_secondary())
                    .small()
            );
        }
    }

    fn show_style_settings(&mut self, ui: &mut Ui) {
        ui.heading("Style Presets");
        ui.add_space(8.0);

        egui::Grid::new("style_grid")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                for (i, style) in ArtworkStyle::all().iter().enumerate() {
                    let selected = self.state.artwork.style == *style;
                    let btn = egui::Button::new(style.display_name())
                        .fill(if selected {
                            self.theme.palette.accent.gamma_multiply(0.3)
                        } else {
                            self.theme.surface_bg()
                        })
                        .min_size(Vec2::new(100.0, 30.0));

                    if ui.add(btn).clicked() {
                        self.state.artwork.style = *style;
                    }

                    if i % 2 == 1 {
                        ui.end_row();
                    }
                }
            });
    }

    fn show_color_settings(&mut self, ui: &mut Ui) {
        ui.heading("Color Palette");
        ui.add_space(8.0);

        // Current palette
        ui.horizontal(|ui| {
            for (i, color) in self.state.artwork.palette.colors.iter().enumerate() {
                let (rect, response) = ui.allocate_exact_size(Vec2::splat(32.0), egui::Sense::click());
                ui.painter().rect_filled(rect, 4.0, *color);
                ui.painter().rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::WHITE));

                if response.clicked() {
                    self.state.status_message = format!("Edit color {}", i + 1);
                }
            }

            if ui.button("+").clicked() && self.state.artwork.palette.colors.len() < 8 {
                self.state.artwork.palette.colors.push(Color32::GRAY);
            }
        });

        ui.add_space(12.0);

        // Palette presets
        ui.label("Presets:");
        ui.horizontal_wrapped(|ui| {
            let presets = [
                ("Sunset", vec![Color32::from_rgb(255, 94, 98), Color32::from_rgb(255, 195, 113), Color32::from_rgb(255, 154, 158)]),
                ("Ocean", vec![Color32::from_rgb(0, 180, 219), Color32::from_rgb(0, 131, 176), Color32::from_rgb(0, 82, 136)]),
                ("Forest", vec![Color32::from_rgb(34, 139, 34), Color32::from_rgb(85, 107, 47), Color32::from_rgb(107, 142, 35)]),
                ("Neon", vec![Color32::from_rgb(255, 0, 128), Color32::from_rgb(0, 255, 255), Color32::from_rgb(128, 0, 255)]),
                ("Mono", vec![Color32::WHITE, Color32::GRAY, Color32::BLACK]),
            ];

            for (name, colors) in presets {
                if ui.button(name).clicked() {
                    self.state.artwork.palette.colors = colors;
                    self.state.artwork.palette.name = name.to_string();
                }
            }
        });

        ui.add_space(12.0);

        if ui.button("🎵 Extract from Audio Mood").clicked() {
            self.state.status_message = "Analyzing audio for color extraction...".to_string();
            self.state.artwork.palette.from_audio = true;
        }
    }

    fn show_typography_settings(&mut self, ui: &mut Ui) {
        ui.heading("Typography");
        ui.add_space(8.0);

        egui::Grid::new("typography_grid")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("Title Size:");
                ui.add(egui::Slider::new(&mut self.state.artwork.typography.title_size, 24.0..=120.0).suffix("px"));
                ui.end_row();

                ui.label("Artist Size:");
                ui.add(egui::Slider::new(&mut self.state.artwork.typography.artist_size, 12.0..=72.0).suffix("px"));
                ui.end_row();

                ui.label("Position:");
                ui.horizontal(|ui| {
                    for pos in [TextPosition::Top, TextPosition::Center, TextPosition::Bottom] {
                        let name = match pos {
                            TextPosition::Top => "Top",
                            TextPosition::Center => "Center",
                            TextPosition::Bottom => "Bottom",
                            TextPosition::Custom => "Custom",
                        };
                        if ui.selectable_label(self.state.artwork.typography.position == pos, name).clicked() {
                            self.state.artwork.typography.position = pos;
                        }
                    }
                });
                ui.end_row();

                ui.label("Alignment:");
                ui.horizontal(|ui| {
                    for align in [TextAlignment::Left, TextAlignment::Center, TextAlignment::Right] {
                        let name = match align {
                            TextAlignment::Left => "Left",
                            TextAlignment::Center => "Center",
                            TextAlignment::Right => "Right",
                        };
                        if ui.selectable_label(self.state.artwork.typography.alignment == align, name).clicked() {
                            self.state.artwork.typography.alignment = align;
                        }
                    }
                });
                ui.end_row();

                ui.label("Effects:");
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.state.artwork.typography.shadow, "Shadow");
                    ui.checkbox(&mut self.state.artwork.typography.outline, "Outline");
                });
                ui.end_row();
            });
    }

    fn show_overlay_settings(&mut self, ui: &mut Ui) {
        ui.heading("Overlays");
        ui.add_space(8.0);

        ui.label("Add visual effects and textures to your artwork.");
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui.button("+ Gradient").clicked() {
                self.state.artwork.overlays.push(ArtworkOverlay {
                    overlay_type: OverlayType::Gradient,
                    opacity: 0.5,
                    blend_mode: BlendMode::Overlay,
                    position: (0.5, 0.5),
                    scale: 1.0,
                });
            }
            if ui.button("+ Noise").clicked() {
                self.state.artwork.overlays.push(ArtworkOverlay {
                    overlay_type: OverlayType::Noise,
                    opacity: 0.1,
                    blend_mode: BlendMode::Overlay,
                    position: (0.5, 0.5),
                    scale: 1.0,
                });
            }
            if ui.button("+ Waveform").clicked() {
                self.state.artwork.overlays.push(ArtworkOverlay {
                    overlay_type: OverlayType::Waveform,
                    opacity: 0.3,
                    blend_mode: BlendMode::Screen,
                    position: (0.5, 0.5),
                    scale: 1.0,
                });
            }
        });

        if !self.state.artwork.overlays.is_empty() {
            ui.add_space(12.0);
            ui.label("Active Overlays:");

            let mut remove_idx = None;
            for (i, overlay) in self.state.artwork.overlays.iter().enumerate() {
                ui.horizontal(|ui| {
                    let name = match overlay.overlay_type {
                        OverlayType::Gradient => "Gradient",
                        OverlayType::Noise => "Noise",
                        OverlayType::Texture => "Texture",
                        OverlayType::Geometric => "Geometric",
                        OverlayType::Waveform => "Waveform",
                        OverlayType::Spectrum => "Spectrum",
                    };
                    ui.label(name);
                    ui.label(format!("{:.0}%", overlay.opacity * 100.0));
                    if ui.small_button("✕").clicked() {
                        remove_idx = Some(i);
                    }
                });
            }

            if let Some(idx) = remove_idx {
                self.state.artwork.overlays.remove(idx);
            }
        }
    }

    fn show_ai_generate(&mut self, ui: &mut Ui) {
        ui.heading("AI Artwork Generation");
        ui.add_space(8.0);

        ui.label("Describe your desired artwork:");
        ui.add_space(4.0);

        ui.add(
            egui::TextEdit::multiline(&mut self.state.artwork.ai_prompt)
                .desired_rows(4)
                .desired_width(f32::INFINITY)
                .hint_text("E.g., 'Abstract album cover with flowing neon lines, dark background, futuristic vibe'")
        );

        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label("Include:");
            ui.checkbox(&mut self.state.artwork.typography.shadow, "Title Text");
        });

        ui.add_space(12.0);

        let can_generate = !self.state.artwork.ai_prompt.is_empty() && !self.state.is_generating;

        if ui.add_enabled(can_generate, egui::Button::new("🤖 Generate Artwork")).clicked() {
            self.state.is_generating = true;
            self.state.generation_progress = 0.0;
            self.state.status_message = "Generating artwork with AI...".to_string();
        }

        if self.state.is_generating && self.state.current_tab == ReleaseTab::Artwork {
            ui.add_space(8.0);
            ui.add(egui::ProgressBar::new(self.state.generation_progress).text("Generating..."));
        }

        ui.add_space(16.0);
        ui.separator();

        ui.label(
            egui::RichText::new("Tips for better results:")
                .color(self.theme.text_secondary())
        );
        ui.label("• Be specific about colors, mood, and style");
        ui.label("• Reference art movements or genres");
        ui.label("• Mention composition and layout preferences");
    }

    // ========================================================================
    // Visualizer Tab
    // ========================================================================

    fn show_visualizer(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Left: Preview
            ui.vertical(|ui| {
                ui.set_min_width(500.0);
                self.show_visualizer_preview(ui);
            });

            ui.separator();

            // Right: Controls
            ScrollArea::vertical()
                .id_salt("visualizer_controls")
                .show(ui, |ui| {
                    ui.set_min_width(350.0);
                    self.show_visualizer_controls(ui);
                });
        });
    }

    fn show_visualizer_preview(&mut self, ui: &mut Ui) {
        ui.heading("Visualizer Preview");
        ui.add_space(8.0);

        // Preview area
        let available = ui.available_size();
        let preview_width = available.x.min(500.0);
        let (w, h) = self.state.visualizer.export.resolution.dimensions();
        let aspect = h as f32 / w as f32;
        let preview_height = (preview_width * aspect).min(300.0);

        let (rect, _response) = ui.allocate_exact_size(
            Vec2::new(preview_width, preview_height),
            egui::Sense::hover()
        );

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, self.state.visualizer.background);

        // Draw visualizer preview based on type
        self.draw_visualizer_preview(&painter, rect);

        ui.add_space(8.0);

        // Playback controls
        ui.horizontal(|ui| {
            if ui.button("▶ Preview").clicked() {
                self.state.status_message = "Playing preview...".to_string();
            }
            if ui.button("⏹ Stop").clicked() {
                self.state.status_message = "Stopped".to_string();
            }
            ui.separator();
            if ui.button("📥 Export Video").clicked() {
                self.state.status_message = "Exporting video...".to_string();
            }
            if ui.button("📥 Export GIF").clicked() {
                self.state.status_message = "Exporting GIF...".to_string();
            }
        });
    }

    fn draw_visualizer_preview(&self, painter: &egui::Painter, rect: Rect) {
        let center = rect.center();

        match self.state.visualizer.viz_type {
            VisualizerType::SpectrumBars => {
                let bar_count = self.state.visualizer.params.bar_count.min(64) as usize;
                let bar_width = rect.width() / (bar_count as f32 * 1.5);
                let spacing = bar_width * 0.5;

                for i in 0..bar_count {
                    let x = rect.left() + (i as f32 * (bar_width + spacing)) + spacing;
                    let height = ((i as f32 * 0.3).sin().abs() + 0.2) * rect.height() * 0.8;
                    let color_idx = i % self.state.visualizer.colors.colors.len();
                    let color = self.state.visualizer.colors.colors.get(color_idx)
                        .copied()
                        .unwrap_or(Color32::WHITE);

                    let bar_rect = Rect::from_min_size(
                        Pos2::new(x, rect.bottom() - height),
                        Vec2::new(bar_width, height)
                    );
                    painter.rect_filled(bar_rect, 2.0, color);
                }
            }
            VisualizerType::Waveform => {
                let points: Vec<Pos2> = (0..100).map(|i| {
                    let x = rect.left() + (i as f32 / 100.0) * rect.width();
                    let y = center.y + (i as f32 * 0.2).sin() * rect.height() * 0.3;
                    Pos2::new(x, y)
                }).collect();

                let color = self.state.visualizer.colors.colors.first()
                    .copied()
                    .unwrap_or(Color32::WHITE);
                painter.add(egui::Shape::line(points, Stroke::new(2.0, color)));
            }
            VisualizerType::CircularSpectrum => {
                let radius = rect.height().min(rect.width()) * 0.35;
                let segments = 32;

                for i in 0..segments {
                    let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
                    let length = radius * (0.5 + (i as f32 * 0.5).sin().abs() * 0.5);
                    let color_idx = i % self.state.visualizer.colors.colors.len();
                    let color = self.state.visualizer.colors.colors.get(color_idx)
                        .copied()
                        .unwrap_or(Color32::WHITE);

                    let start = Pos2::new(
                        center.x + angle.cos() * radius * 0.3,
                        center.y + angle.sin() * radius * 0.3
                    );
                    let end = Pos2::new(
                        center.x + angle.cos() * length,
                        center.y + angle.sin() * length
                    );
                    painter.line_segment([start, end], Stroke::new(3.0, color));
                }
            }
            VisualizerType::Particles => {
                // Draw scattered particles
                let count = (self.state.visualizer.params.particle_count / 10).min(100);
                for i in 0..count {
                    let x = rect.left() + (i as f32 * 17.0) % rect.width();
                    let y = rect.top() + (i as f32 * 23.0) % rect.height();
                    let size = 2.0 + (i as f32 * 0.1).sin().abs() * 4.0;
                    let color_idx = i as usize % self.state.visualizer.colors.colors.len();
                    let color = self.state.visualizer.colors.colors.get(color_idx)
                        .copied()
                        .unwrap_or(Color32::WHITE);
                    painter.circle_filled(Pos2::new(x, y), size, color);
                }
            }
            _ => {
                // Default: simple centered visual
                let color = self.state.visualizer.colors.colors.first()
                    .copied()
                    .unwrap_or(Color32::WHITE);
                painter.circle_stroke(center, 50.0, Stroke::new(2.0, color));
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    self.state.visualizer.viz_type.display_name(),
                    egui::FontId::proportional(14.0),
                    color,
                );
            }
        }
    }

    fn show_visualizer_controls(&mut self, ui: &mut Ui) {
        // Sub-tabs
        ui.horizontal(|ui| {
            for tab in [VisualizerTab::Type, VisualizerTab::Colors, VisualizerTab::Parameters, VisualizerTab::Export] {
                let name = match tab {
                    VisualizerTab::Preview => "Preview",
                    VisualizerTab::Type => "Type",
                    VisualizerTab::Colors => "Colors",
                    VisualizerTab::Parameters => "Params",
                    VisualizerTab::Export => "Export",
                };
                if ui.selectable_label(self.state.visualizer_tab == tab, name).clicked() {
                    self.state.visualizer_tab = tab;
                }
            }
        });

        ui.separator();

        match self.state.visualizer_tab {
            VisualizerTab::Preview => {}
            VisualizerTab::Type => self.show_visualizer_type_settings(ui),
            VisualizerTab::Colors => self.show_visualizer_color_settings(ui),
            VisualizerTab::Parameters => self.show_visualizer_params(ui),
            VisualizerTab::Export => self.show_visualizer_export(ui),
        }
    }

    fn show_visualizer_type_settings(&mut self, ui: &mut Ui) {
        ui.heading("Visualizer Type");
        ui.add_space(8.0);

        egui::Grid::new("viz_type_grid")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                for (i, viz_type) in VisualizerType::all().iter().enumerate() {
                    let selected = self.state.visualizer.viz_type == *viz_type;
                    let btn = egui::Button::new(viz_type.display_name())
                        .fill(if selected {
                            self.theme.palette.accent.gamma_multiply(0.3)
                        } else {
                            self.theme.surface_bg()
                        })
                        .min_size(Vec2::new(120.0, 30.0));

                    if ui.add(btn).clicked() {
                        self.state.visualizer.viz_type = *viz_type;
                    }

                    if i % 2 == 1 {
                        ui.end_row();
                    }
                }
            });
    }

    fn show_visualizer_color_settings(&mut self, ui: &mut Ui) {
        ui.heading("Colors");
        ui.add_space(8.0);

        ui.label("Background:");
        let mut bg = self.state.visualizer.background;
        if ui.color_edit_button_srgba(&mut bg).changed() {
            self.state.visualizer.background = bg;
        }

        ui.add_space(12.0);

        ui.label("Visualizer Colors:");
        ui.horizontal(|ui| {
            for color in &self.state.visualizer.colors.colors {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(24.0), egui::Sense::click());
                ui.painter().rect_filled(rect, 2.0, *color);
            }
        });

        ui.add_space(8.0);
        if ui.button("Use Artwork Palette").clicked() {
            self.state.visualizer.colors = self.state.artwork.palette.clone();
        }
    }

    fn show_visualizer_params(&mut self, ui: &mut Ui) {
        ui.heading("Parameters");
        ui.add_space(8.0);

        egui::Grid::new("viz_params_grid")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("Sensitivity:");
                ui.add(egui::Slider::new(&mut self.state.visualizer.sensitivity, 0.1..=3.0));
                ui.end_row();

                ui.label("Smoothing:");
                ui.add(egui::Slider::new(&mut self.state.visualizer.smoothing, 0.0..=1.0));
                ui.end_row();

                match self.state.visualizer.viz_type {
                    VisualizerType::SpectrumBars => {
                        ui.label("Bar Count:");
                        ui.add(egui::Slider::new(&mut self.state.visualizer.params.bar_count, 8..=128));
                        ui.end_row();
                    }
                    VisualizerType::Particles => {
                        ui.label("Particle Count:");
                        ui.add(egui::Slider::new(&mut self.state.visualizer.params.particle_count, 100..=5000));
                        ui.end_row();
                    }
                    _ => {}
                }

                ui.label("Glow:");
                ui.add(egui::Slider::new(&mut self.state.visualizer.params.glow, 0.0..=1.0));
                ui.end_row();

                ui.label("Mirror:");
                ui.checkbox(&mut self.state.visualizer.params.mirror, "");
                ui.end_row();
            });
    }

    fn show_visualizer_export(&mut self, ui: &mut Ui) {
        ui.heading("Export Settings");
        ui.add_space(8.0);

        egui::Grid::new("viz_export_grid")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("Resolution:");
                egui::ComboBox::from_id_salt("viz_resolution")
                    .selected_text(self.state.visualizer.export.resolution.display_name())
                    .show_ui(ui, |ui| {
                        for res in [VideoResolution::HD720, VideoResolution::HD1080, VideoResolution::UHD4K, VideoResolution::Square1080, VideoResolution::Portrait] {
                            ui.selectable_value(&mut self.state.visualizer.export.resolution, res, res.display_name());
                        }
                    });
                ui.end_row();

                ui.label("Frame Rate:");
                ui.horizontal(|ui| {
                    for fps in [24, 30, 60] {
                        if ui.selectable_label(self.state.visualizer.export.fps == fps, format!("{}fps", fps)).clicked() {
                            self.state.visualizer.export.fps = fps;
                        }
                    }
                });
                ui.end_row();

                ui.label("Quality:");
                ui.add(egui::Slider::new(&mut self.state.visualizer.export.quality, 50..=100).suffix("%"));
                ui.end_row();

                ui.label("Include Audio:");
                ui.checkbox(&mut self.state.visualizer.export.include_audio, "");
                ui.end_row();
            });
    }

    // ========================================================================
    // Social Media Tab
    // ========================================================================

    fn show_social_media(&mut self, ui: &mut Ui) {
        ui.heading("Social Media Assets");
        ui.add_space(8.0);

        ui.label("Generate platform-optimized promotional graphics.");
        ui.add_space(12.0);

        // Platform selection
        ui.group(|ui| {
            ui.label("Select Platforms:");
            ui.add_space(4.0);

            egui::Grid::new("platform_selection")
                .num_columns(4)
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    for (i, platform) in SocialPlatform::all().iter().enumerate() {
                        let selected = self.state.selected_platforms.contains(platform);
                        if ui.checkbox(&mut selected.clone(), platform.display_name()).changed() {
                            if selected {
                                self.state.selected_platforms.retain(|p| p != platform);
                            } else {
                                self.state.selected_platforms.push(*platform);
                            }
                        }
                        if (i + 1) % 4 == 0 {
                            ui.end_row();
                        }
                    }
                });
        });

        ui.add_space(12.0);

        // Generate button
        ui.horizontal(|ui| {
            if ui.button("🎨 Generate All Assets").clicked() {
                self.state.init_promo_assets();
                self.state.status_message = "Generating social media assets...".to_string();
            }

            if !self.state.promo_assets.is_empty() {
                if ui.button("📥 Export All").clicked() {
                    self.state.status_message = "Exporting assets...".to_string();
                }
            }
        });

        ui.add_space(16.0);

        // Generated assets preview
        if !self.state.promo_assets.is_empty() {
            ui.heading("Generated Assets");
            ui.add_space(8.0);

            egui::Grid::new("assets_preview")
                .num_columns(3)
                .spacing([16.0, 16.0])
                .show(ui, |ui| {
                    for (i, asset) in self.state.promo_assets.iter().enumerate() {
                        ui.vertical(|ui| {
                            let (w, h) = asset.platform.dimensions();
                            let aspect = h as f32 / w as f32;
                            let preview_w = 150.0;
                            let preview_h = preview_w * aspect.min(1.5);

                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::new(preview_w, preview_h),
                                egui::Sense::hover()
                            );

                            let painter = ui.painter_at(rect);
                            painter.rect_filled(rect, 4.0, self.theme.surface_bg());

                            // Draw mini preview
                            if let Some(color) = self.state.artwork.palette.colors.first() {
                                painter.rect_filled(rect.shrink(2.0), 2.0, *color);
                            }

                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                &self.state.release_title,
                                egui::FontId::proportional(10.0),
                                Color32::WHITE,
                            );

                            ui.label(asset.platform.display_name());
                            ui.label(
                                egui::RichText::new(format!("{}×{}", w, h))
                                    .color(self.theme.text_secondary())
                                    .small()
                            );
                        });

                        if (i + 1) % 3 == 0 {
                            ui.end_row();
                        }
                    }
                });
        }
    }

    // ========================================================================
    // Press Kit Tab
    // ========================================================================

    fn show_press_kit(&mut self, ui: &mut Ui) {
        ui.heading("Electronic Press Kit (EPK)");
        ui.add_space(8.0);

        ScrollArea::vertical()
            .id_salt("press_kit_scroll")
            .show(ui, |ui| {
                // Bio section
                ui.group(|ui| {
                    ui.heading("Artist Bio");
                    ui.add_space(4.0);

                    ui.label("Short Bio (1-2 sentences):");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.state.press_kit.bio_short)
                            .desired_rows(2)
                            .desired_width(f32::INFINITY)
                    );

                    ui.add_space(8.0);

                    ui.label("Full Bio:");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.state.press_kit.bio_long)
                            .desired_rows(6)
                            .desired_width(f32::INFINITY)
                    );
                });

                ui.add_space(12.0);

                // Press release
                ui.group(|ui| {
                    ui.heading("Press Release");
                    ui.add_space(4.0);

                    ui.add(
                        egui::TextEdit::multiline(&mut self.state.press_kit.press_release)
                            .desired_rows(8)
                            .desired_width(f32::INFINITY)
                            .hint_text("Write your press release here...")
                    );
                });

                ui.add_space(12.0);

                // Contact info
                ui.group(|ui| {
                    ui.heading("Contact Information");
                    ui.add_space(4.0);

                    egui::Grid::new("contact_grid")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Press Contact:");
                            ui.text_edit_singleline(&mut self.state.press_kit.contact_email);
                            ui.end_row();

                            ui.label("Management:");
                            ui.text_edit_singleline(&mut self.state.press_kit.management);
                            ui.end_row();

                            ui.label("Booking:");
                            ui.text_edit_singleline(&mut self.state.press_kit.booking);
                            ui.end_row();
                        });
                });

                ui.add_space(12.0);

                // Social links
                ui.group(|ui| {
                    ui.heading("Social Links");
                    ui.add_space(4.0);

                    egui::Grid::new("social_grid")
                        .num_columns(4)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Website:");
                            ui.text_edit_singleline(&mut self.state.press_kit.social_links.website);
                            ui.label("Instagram:");
                            ui.text_edit_singleline(&mut self.state.press_kit.social_links.instagram);
                            ui.end_row();

                            ui.label("Twitter:");
                            ui.text_edit_singleline(&mut self.state.press_kit.social_links.twitter);
                            ui.label("YouTube:");
                            ui.text_edit_singleline(&mut self.state.press_kit.social_links.youtube);
                            ui.end_row();

                            ui.label("Spotify:");
                            ui.text_edit_singleline(&mut self.state.press_kit.social_links.spotify);
                            ui.label("Apple Music:");
                            ui.text_edit_singleline(&mut self.state.press_kit.social_links.apple_music);
                            ui.end_row();
                        });
                });

                ui.add_space(12.0);

                // Export
                ui.horizontal(|ui| {
                    if ui.button("📥 Export EPK as PDF").clicked() {
                        self.state.status_message = "Generating EPK PDF...".to_string();
                    }
                    if ui.button("📥 Export EPK as ZIP").clicked() {
                        self.state.status_message = "Creating EPK archive...".to_string();
                    }
                });
            });
    }

    // ========================================================================
    // Merch Tab
    // ========================================================================

    fn show_merch(&mut self, ui: &mut Ui) {
        ui.heading("Merchandise Mockups");
        ui.add_space(8.0);

        ui.label("Preview your artwork on merchandise.");
        ui.add_space(12.0);

        // Product selector
        ui.horizontal(|ui| {
            ui.label("Add Product:");
            for product in [MerchType::TShirt, MerchType::Vinyl, MerchType::Poster, MerchType::CD] {
                if ui.button(product.display_name()).clicked() {
                    self.state.merch_mockups.push(MerchMockup {
                        product,
                        color: Color32::BLACK,
                        placement: MerchPlacement::Center,
                        scale: 0.6,
                        preview_path: None,
                    });
                }
            }
        });

        ui.add_space(16.0);

        // Mockup grid
        if self.state.merch_mockups.is_empty() {
            ui.label(
                egui::RichText::new("No mockups yet. Add a product above to get started.")
                    .color(self.theme.text_secondary())
            );
        } else {
            let mut remove_idx = None;

            egui::Grid::new("merch_grid")
                .num_columns(3)
                .spacing([16.0, 16.0])
                .show(ui, |ui| {
                    for (i, mockup) in self.state.merch_mockups.iter().enumerate() {
                        ui.vertical(|ui| {
                            // Preview
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::new(150.0, 180.0),
                                egui::Sense::hover()
                            );

                            let painter = ui.painter_at(rect);
                            painter.rect_filled(rect, 4.0, self.theme.surface_bg());

                            // Draw product outline
                            match mockup.product {
                                MerchType::TShirt => {
                                    // T-shirt shape
                                    let shirt_rect = rect.shrink(20.0);
                                    painter.rect_filled(shirt_rect, 8.0, mockup.color);

                                    // Artwork area
                                    let art_rect = Rect::from_center_size(
                                        shirt_rect.center() - Vec2::new(0.0, 10.0),
                                        Vec2::splat(60.0 * mockup.scale)
                                    );
                                    if let Some(color) = self.state.artwork.palette.colors.first() {
                                        painter.rect_filled(art_rect, 2.0, *color);
                                    }
                                }
                                MerchType::Vinyl => {
                                    // Vinyl record
                                    let center = rect.center();
                                    painter.circle_filled(center, 60.0, Color32::from_rgb(20, 20, 20));
                                    painter.circle_stroke(center, 60.0, Stroke::new(2.0, Color32::from_rgb(50, 50, 50)));

                                    // Label
                                    if let Some(color) = self.state.artwork.palette.colors.first() {
                                        painter.circle_filled(center, 25.0, *color);
                                    }
                                    painter.circle_filled(center, 5.0, Color32::WHITE);
                                }
                                MerchType::Poster | MerchType::CD => {
                                    // Square artwork
                                    let art_rect = Rect::from_center_size(rect.center(), Vec2::splat(100.0));
                                    if let Some(color) = self.state.artwork.palette.colors.first() {
                                        painter.rect_filled(art_rect, 4.0, *color);
                                    }
                                    painter.text(
                                        art_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        &self.state.release_title,
                                        egui::FontId::proportional(10.0),
                                        Color32::WHITE,
                                    );
                                }
                                _ => {
                                    painter.text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        mockup.product.display_name(),
                                        egui::FontId::proportional(12.0),
                                        self.theme.text_primary(),
                                    );
                                }
                            }

                            ui.label(mockup.product.display_name());

                            ui.horizontal(|ui| {
                                if ui.small_button("📥 Export").clicked() {
                                    self.state.status_message = format!("Exporting {} mockup...", mockup.product.display_name());
                                }
                                if ui.small_button("✕").clicked() {
                                    remove_idx = Some(i);
                                }
                            });
                        });

                        if (i + 1) % 3 == 0 {
                            ui.end_row();
                        }
                    }
                });

            if let Some(idx) = remove_idx {
                self.state.merch_mockups.remove(idx);
            }
        }
    }
}
