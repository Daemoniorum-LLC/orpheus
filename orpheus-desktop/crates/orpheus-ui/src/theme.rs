//! Theme configuration for Orpheus
//!
//! Extends the daemoniorum-egui theme system with Orpheus-specific customizations
//! for music production workflows.

use daemoniorum_egui::theme::{Theme as DaemoniorumTheme, ThemeMode};
use egui::{Color32, Context, Rounding, Stroke};

/// Orpheus-specific color palette
///
/// Extends the Daemoniorum palette with colors optimized for music production,
/// including track type colors, meter colors, and timeline accents.
#[derive(Debug, Clone)]
pub struct OrpheusColorPalette {
    // Base backgrounds
    /// Primary background (main window)
    pub bg_primary: Color32,
    /// Secondary background (panels)
    pub bg_secondary: Color32,
    /// Tertiary background (cards, controls)
    pub bg_tertiary: Color32,

    // Text
    /// Primary text
    pub text_primary: Color32,
    /// Secondary text
    pub text_secondary: Color32,
    /// Muted text
    pub text_muted: Color32,

    // Accent (Phthalo Green - Orpheus brand)
    /// Primary accent color
    pub accent: Color32,
    /// Accent hover state
    pub accent_hover: Color32,
    /// Accent active/pressed state
    pub accent_active: Color32,

    // Status
    /// Success color
    pub success: Color32,
    /// Warning color
    pub warning: Color32,
    /// Error color
    pub error: Color32,

    // Borders
    /// Border color
    pub border: Color32,
    /// Border light
    pub border_light: Color32,

    // Track type colors (music-specific)
    /// Guitar track color
    pub track_guitar: Color32,
    /// Bass track color
    pub track_bass: Color32,
    /// Drums track color
    pub track_drums: Color32,
    /// Vocal track color
    pub track_vocals: Color32,
    /// Keys/Synth track color
    pub track_keys: Color32,
    /// Aux/Bus track color
    pub track_aux: Color32,

    // Meter colors
    /// Meter green (safe level)
    pub meter_green: Color32,
    /// Meter yellow (caution)
    pub meter_yellow: Color32,
    /// Meter red (clipping)
    pub meter_red: Color32,

    // Timeline
    /// Playhead color
    pub playhead: Color32,
    /// Beat grid color
    pub beat_grid: Color32,
    /// Bar grid color
    pub bar_grid: Color32,
    /// Selection highlight
    pub selection: Color32,

    // Accessibility
    /// Focus ring color for keyboard navigation
    pub focus_ring: Color32,
}

impl OrpheusColorPalette {
    /// Create dark theme palette
    pub fn dark() -> Self {
        Self {
            // Backgrounds - slightly warmer than pure blacks
            bg_primary: Color32::from_rgb(18, 18, 24),
            bg_secondary: Color32::from_rgb(26, 26, 36),
            bg_tertiary: Color32::from_rgb(36, 36, 48),

            // Text
            text_primary: Color32::from_rgb(230, 230, 240),
            text_secondary: Color32::from_rgb(140, 140, 160),
            text_muted: Color32::from_rgb(90, 90, 110),

            // Accent - Phthalo Green
            accent: Color32::from_rgb(26, 123, 93),
            accent_hover: Color32::from_rgb(36, 153, 113),
            accent_active: Color32::from_rgb(20, 100, 75),

            // Status
            success: Color32::from_rgb(52, 168, 83),
            warning: Color32::from_rgb(251, 188, 4),
            error: Color32::from_rgb(234, 67, 53),

            // Borders
            border: Color32::from_rgb(50, 50, 65),
            border_light: Color32::from_rgb(70, 70, 90),

            // Track colors - vibrant but not distracting
            track_guitar: Color32::from_rgb(66, 133, 244),  // Blue
            track_bass: Color32::from_rgb(156, 39, 176),    // Purple
            track_drums: Color32::from_rgb(255, 152, 0),    // Orange
            track_vocals: Color32::from_rgb(233, 30, 99),   // Pink
            track_keys: Color32::from_rgb(0, 188, 212),     // Cyan
            track_aux: Color32::from_rgb(158, 158, 158),    // Gray

            // Meters
            meter_green: Color32::from_rgb(76, 175, 80),
            meter_yellow: Color32::from_rgb(255, 235, 59),
            meter_red: Color32::from_rgb(244, 67, 54),

            // Timeline
            playhead: Color32::from_rgb(255, 255, 255),
            beat_grid: Color32::from_rgb(50, 50, 65),
            bar_grid: Color32::from_rgb(70, 70, 90),
            selection: Color32::from_rgba_unmultiplied(26, 123, 93, 80),

            // Accessibility - bright focus ring visible on dark backgrounds
            focus_ring: Color32::from_rgb(100, 200, 255),
        }
    }

    /// Create light theme palette
    pub fn light() -> Self {
        Self {
            bg_primary: Color32::from_rgb(250, 250, 252),
            bg_secondary: Color32::from_rgb(240, 240, 245),
            bg_tertiary: Color32::from_rgb(255, 255, 255),

            text_primary: Color32::from_rgb(30, 30, 40),
            text_secondary: Color32::from_rgb(100, 100, 120),
            text_muted: Color32::from_rgb(150, 150, 170),

            accent: Color32::from_rgb(26, 123, 93),
            accent_hover: Color32::from_rgb(20, 100, 75),
            accent_active: Color32::from_rgb(15, 80, 60),

            success: Color32::from_rgb(46, 125, 50),
            warning: Color32::from_rgb(237, 137, 54),
            error: Color32::from_rgb(211, 47, 47),

            border: Color32::from_rgb(200, 200, 210),
            border_light: Color32::from_rgb(220, 220, 230),

            track_guitar: Color32::from_rgb(33, 100, 200),
            track_bass: Color32::from_rgb(123, 31, 139),
            track_drums: Color32::from_rgb(230, 126, 0),
            track_vocals: Color32::from_rgb(194, 24, 82),
            track_keys: Color32::from_rgb(0, 150, 170),
            track_aux: Color32::from_rgb(117, 117, 117),

            meter_green: Color32::from_rgb(56, 142, 60),
            meter_yellow: Color32::from_rgb(245, 200, 35),
            meter_red: Color32::from_rgb(211, 47, 47),

            playhead: Color32::from_rgb(26, 123, 93),
            beat_grid: Color32::from_rgb(220, 220, 230),
            bar_grid: Color32::from_rgb(180, 180, 200),
            selection: Color32::from_rgba_unmultiplied(26, 123, 93, 50),

            // Accessibility - darker focus ring visible on light backgrounds
            focus_ring: Color32::from_rgb(0, 100, 200),
        }
    }

    /// Create high-contrast theme palette for accessibility
    pub fn high_contrast() -> Self {
        Self {
            // Pure black/white backgrounds for maximum contrast
            bg_primary: Color32::from_rgb(0, 0, 0),
            bg_secondary: Color32::from_rgb(15, 15, 15),
            bg_tertiary: Color32::from_rgb(30, 30, 30),

            // Pure white text for maximum readability
            text_primary: Color32::WHITE,
            text_secondary: Color32::from_rgb(220, 220, 220),
            text_muted: Color32::from_rgb(180, 180, 180),

            // Brighter accent for visibility
            accent: Color32::from_rgb(0, 255, 170),
            accent_hover: Color32::from_rgb(50, 255, 200),
            accent_active: Color32::from_rgb(0, 200, 140),

            // High-visibility status colors
            success: Color32::from_rgb(0, 255, 100),
            warning: Color32::from_rgb(255, 255, 0),
            error: Color32::from_rgb(255, 50, 50),

            // Strong borders
            border: Color32::from_rgb(100, 100, 100),
            border_light: Color32::from_rgb(150, 150, 150),

            // Saturated track colors for distinction
            track_guitar: Color32::from_rgb(100, 180, 255),
            track_bass: Color32::from_rgb(200, 100, 255),
            track_drums: Color32::from_rgb(255, 180, 50),
            track_vocals: Color32::from_rgb(255, 100, 150),
            track_keys: Color32::from_rgb(0, 255, 255),
            track_aux: Color32::from_rgb(200, 200, 200),

            // Bright meters
            meter_green: Color32::from_rgb(0, 255, 100),
            meter_yellow: Color32::from_rgb(255, 255, 0),
            meter_red: Color32::from_rgb(255, 50, 50),

            // High-visibility timeline
            playhead: Color32::from_rgb(255, 255, 0),
            beat_grid: Color32::from_rgb(60, 60, 60),
            bar_grid: Color32::from_rgb(100, 100, 100),
            selection: Color32::from_rgba_unmultiplied(0, 255, 170, 100),

            // Accessibility - very bright focus ring for maximum visibility
            focus_ring: Color32::from_rgb(255, 255, 0),
        }
    }
}

impl Default for OrpheusColorPalette {
    fn default() -> Self {
        Self::dark()
    }
}

// Legacy compatibility alias
/// Color palette alias (legacy)
pub type ColorPalette = OrpheusColorPalette;

/// Orpheus theme that extends Daemoniorum theme
#[derive(Debug, Clone)]
pub struct OrpheusTheme {
    /// Base Daemoniorum theme
    pub base: DaemoniorumTheme,
    /// Orpheus-specific color palette
    pub palette: OrpheusColorPalette,
    /// Enable rounded corners (DAW-style)
    pub rounded_controls: bool,
}

impl OrpheusTheme {
    /// Create from base theme with custom accent
    pub fn from_base(base: DaemoniorumTheme) -> Self {
        let palette = match base.mode {
            ThemeMode::Dark => OrpheusColorPalette::dark(),
            ThemeMode::Light => OrpheusColorPalette::light(),
            ThemeMode::HighContrast => OrpheusColorPalette::high_contrast(),
        };

        Self {
            base,
            palette,
            rounded_controls: true, // DAWs typically use rounded controls
        }
    }

    /// Create dark Orpheus theme
    pub fn dark() -> Self {
        Self::from_base(DaemoniorumTheme::dark())
    }

    /// Create light Orpheus theme
    pub fn light() -> Self {
        Self::from_base(DaemoniorumTheme::light())
    }

    /// Create high-contrast Orpheus theme for accessibility
    pub fn high_contrast() -> Self {
        Self {
            base: DaemoniorumTheme::high_contrast(),
            palette: OrpheusColorPalette::high_contrast(),
            rounded_controls: true,
        }
    }

    /// Check if this is a high-contrast theme
    pub fn is_high_contrast(&self) -> bool {
        self.base.mode == ThemeMode::HighContrast
    }

    /// Set custom accent color
    pub fn with_accent(mut self, accent: Color32) -> Self {
        self.palette.accent = accent;
        // Compute hover/active variants
        self.palette.accent_hover = lighten(accent, 0.2);
        self.palette.accent_active = darken(accent, 0.2);
        self
    }

    /// Get accent color
    pub fn accent_color(&self) -> Color32 {
        self.palette.accent
    }

    /// Enable/disable rounded controls
    pub fn with_rounded_controls(mut self, rounded: bool) -> Self {
        self.rounded_controls = rounded;
        self
    }

    /// Apply theme to egui context
    pub fn apply(&self, ctx: &Context) {
        // Start with base Daemoniorum theme
        self.base.apply(ctx);

        // Apply Orpheus customizations
        let mut style = (*ctx.style()).clone();
        let p = &self.palette;

        let mut visuals = style.visuals.clone();

        // Window styling
        visuals.window_fill = p.bg_secondary;
        visuals.window_stroke = Stroke::new(1.0, p.border);

        // Panel styling
        visuals.panel_fill = p.bg_secondary;

        // Extreme backgrounds
        visuals.extreme_bg_color = p.bg_primary;
        visuals.faint_bg_color = p.bg_tertiary;

        // Widget styling
        visuals.widgets.noninteractive.bg_fill = p.bg_tertiary;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.text_secondary);

        visuals.widgets.inactive.bg_fill = p.bg_tertiary;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, p.text_primary);

        visuals.widgets.hovered.bg_fill = Color32::from_rgb(50, 50, 65);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, p.text_primary);

        visuals.widgets.active.bg_fill = p.accent;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);

        // Rounding
        let rounding = if self.rounded_controls {
            Rounding::same(4.0)
        } else {
            Rounding::ZERO
        };

        visuals.widgets.noninteractive.rounding = rounding;
        visuals.widgets.inactive.rounding = rounding;
        visuals.widgets.hovered.rounding = rounding;
        visuals.widgets.active.rounding = rounding;
        visuals.window_rounding = rounding;

        // Selection
        visuals.selection.bg_fill = p.selection;
        visuals.selection.stroke = Stroke::new(1.0, p.accent);

        // Text
        visuals.override_text_color = Some(p.text_primary);

        // Focus ring for keyboard navigation (accessibility)
        // Apply to all widget states when focused
        let focus_width = if self.is_high_contrast() { 3.0 } else { 2.0 };
        visuals.widgets.hovered.expansion = 0.0;
        visuals.widgets.active.expansion = 0.0;

        // The selection stroke is used for keyboard focus
        visuals.selection.stroke = Stroke::new(focus_width, p.focus_ring);

        style.visuals = visuals;

        // Spacing
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.window_margin = egui::Margin::same(12.0);

        ctx.set_style(style);
    }

    // Color accessors for UI components

    /// Get accent color
    pub fn accent(&self) -> Color32 {
        self.palette.accent
    }

    /// Get success color
    pub fn success(&self) -> Color32 {
        self.palette.success
    }

    /// Get warning color
    pub fn warning(&self) -> Color32 {
        self.palette.warning
    }

    /// Get error color
    pub fn error(&self) -> Color32 {
        self.palette.error
    }

    /// Panel background color
    pub fn panel_bg(&self) -> Color32 {
        self.palette.bg_secondary
    }

    /// Surface background color
    pub fn surface_bg(&self) -> Color32 {
        self.palette.bg_tertiary
    }

    /// Primary text color
    pub fn text_primary(&self) -> Color32 {
        self.palette.text_primary
    }

    /// Secondary text color
    pub fn text_secondary(&self) -> Color32 {
        self.palette.text_secondary
    }

    /// Muted text color
    pub fn text_muted(&self) -> Color32 {
        self.palette.text_muted
    }

    /// Border color
    pub fn border(&self) -> Color32 {
        self.palette.border
    }

    /// Get color for track type
    pub fn track_color(&self, track_type: &str) -> Color32 {
        match track_type.to_lowercase().as_str() {
            "guitar" => self.palette.track_guitar,
            "bass" => self.palette.track_bass,
            "drums" | "drum" => self.palette.track_drums,
            "vocals" | "vocal" | "voice" => self.palette.track_vocals,
            "keys" | "keyboard" | "synth" | "piano" => self.palette.track_keys,
            "aux" | "bus" | "send" => self.palette.track_aux,
            _ => self.palette.accent,
        }
    }

    /// Get meter color based on level (0.0 - 1.0)
    pub fn meter_color(&self, level: f32) -> Color32 {
        if level > 0.95 {
            self.palette.meter_red
        } else if level > 0.7 {
            self.palette.meter_yellow
        } else {
            self.palette.meter_green
        }
    }

    /// Get playhead color
    pub fn playhead(&self) -> Color32 {
        self.palette.playhead
    }

    /// Get beat grid color
    pub fn beat_grid(&self) -> Color32 {
        self.palette.beat_grid
    }

    /// Get bar grid color
    pub fn bar_grid(&self) -> Color32 {
        self.palette.bar_grid
    }

    /// Get focus ring color for keyboard navigation
    pub fn focus_ring(&self) -> Color32 {
        self.palette.focus_ring
    }
}

impl Default for OrpheusTheme {
    fn default() -> Self {
        Self::dark()
    }
}

// Legacy type alias for backward compatibility
/// Theme type (alias for OrpheusTheme)
pub type Theme = OrpheusTheme;

// Helper functions

/// Lighten a color by factor (0.0 - 1.0)
fn lighten(color: Color32, factor: f32) -> Color32 {
    let [r, g, b, a] = color.to_array();
    Color32::from_rgba_unmultiplied(
        (r as f32 + (255.0 - r as f32) * factor) as u8,
        (g as f32 + (255.0 - g as f32) * factor) as u8,
        (b as f32 + (255.0 - b as f32) * factor) as u8,
        a,
    )
}

/// Darken a color by factor (0.0 - 1.0)
fn darken(color: Color32, factor: f32) -> Color32 {
    let [r, g, b, a] = color.to_array();
    Color32::from_rgba_unmultiplied(
        (r as f32 * (1.0 - factor)) as u8,
        (g as f32 * (1.0 - factor)) as u8,
        (b as f32 * (1.0 - factor)) as u8,
        a,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orpheus_theme_dark() {
        let theme = OrpheusTheme::dark();
        assert!(matches!(theme.base.mode, ThemeMode::Dark));
        // Phthalo Green accent
        assert_eq!(theme.palette.accent, Color32::from_rgb(26, 123, 93));
    }

    #[test]
    fn test_orpheus_theme_light() {
        let theme = OrpheusTheme::light();
        assert!(matches!(theme.base.mode, ThemeMode::Light));
    }

    #[test]
    fn test_orpheus_theme_with_accent() {
        let custom_accent = Color32::from_rgb(0, 200, 100);
        let theme = OrpheusTheme::dark().with_accent(custom_accent);

        assert_eq!(theme.accent_color(), custom_accent);
        assert_eq!(theme.palette.accent, custom_accent);
    }

    #[test]
    fn test_theme_apply() {
        let theme = OrpheusTheme::dark();
        let ctx = egui::Context::default();

        theme.apply(&ctx);

        let visuals = ctx.style().visuals.clone();
        assert!(visuals.dark_mode);
    }

    #[test]
    fn test_track_colors() {
        let theme = OrpheusTheme::dark();

        assert_eq!(theme.track_color("guitar"), theme.palette.track_guitar);
        assert_eq!(theme.track_color("BASS"), theme.palette.track_bass);
        assert_eq!(theme.track_color("Drums"), theme.palette.track_drums);
        assert_eq!(theme.track_color("vocal"), theme.palette.track_vocals);
        assert_eq!(theme.track_color("keys"), theme.palette.track_keys);
        assert_eq!(theme.track_color("bus"), theme.palette.track_aux);
        // Unknown tracks get accent color
        assert_eq!(theme.track_color("unknown"), theme.palette.accent);
    }

    #[test]
    fn test_meter_colors() {
        let theme = OrpheusTheme::dark();

        // Low level = green
        assert_eq!(theme.meter_color(0.5), theme.palette.meter_green);
        // Medium level = yellow
        assert_eq!(theme.meter_color(0.8), theme.palette.meter_yellow);
        // High level = red
        assert_eq!(theme.meter_color(0.98), theme.palette.meter_red);
    }

    #[test]
    fn test_rounded_controls() {
        let theme = OrpheusTheme::dark().with_rounded_controls(false);
        assert!(!theme.rounded_controls);

        let theme = OrpheusTheme::dark().with_rounded_controls(true);
        assert!(theme.rounded_controls);
    }

    #[test]
    fn test_color_accessors() {
        let theme = OrpheusTheme::dark();

        assert_eq!(theme.accent(), theme.palette.accent);
        assert_eq!(theme.success(), theme.palette.success);
        assert_eq!(theme.warning(), theme.palette.warning);
        assert_eq!(theme.error(), theme.palette.error);
        assert_eq!(theme.panel_bg(), theme.palette.bg_secondary);
        assert_eq!(theme.text_primary(), theme.palette.text_primary);
    }

    #[test]
    fn test_lighten_darken() {
        let color = Color32::from_rgb(100, 100, 100);

        let lighter = lighten(color, 0.5);
        assert!(lighter.r() > color.r());
        assert!(lighter.g() > color.g());
        assert!(lighter.b() > color.b());

        let darker = darken(color, 0.5);
        assert!(darker.r() < color.r());
        assert!(darker.g() < color.g());
        assert!(darker.b() < color.b());
    }

    #[test]
    fn test_palette_defaults() {
        let dark = OrpheusColorPalette::dark();
        let light = OrpheusColorPalette::light();

        // Dark should have darker backgrounds
        assert!(dark.bg_primary.r() < light.bg_primary.r());

        // Both should use same accent
        assert_eq!(dark.accent, light.accent);
    }

    #[test]
    fn test_legacy_type_alias() {
        // Theme should be an alias for OrpheusTheme
        let theme: Theme = Theme::dark();
        assert!(matches!(theme.base.mode, ThemeMode::Dark));
    }

    #[test]
    fn test_high_contrast_theme() {
        let theme = OrpheusTheme::high_contrast();

        // Should be high contrast mode
        assert!(matches!(theme.base.mode, ThemeMode::HighContrast));
        assert!(theme.is_high_contrast());

        // Should have pure black background
        assert_eq!(theme.palette.bg_primary, Color32::from_rgb(0, 0, 0));

        // Should have pure white text
        assert_eq!(theme.palette.text_primary, Color32::WHITE);

        // Accent should be bright for visibility
        assert!(theme.palette.accent.g() > 200);
    }

    #[test]
    fn test_high_contrast_palette() {
        let hc = OrpheusColorPalette::high_contrast();
        let dark = OrpheusColorPalette::dark();

        // High contrast should have more extreme values
        assert!(hc.bg_primary.r() < dark.bg_primary.r()); // Darker background
        assert!(hc.text_primary.r() > dark.text_primary.r()); // Brighter text
        assert!(hc.meter_green.g() > dark.meter_green.g()); // Brighter meters
    }
}
