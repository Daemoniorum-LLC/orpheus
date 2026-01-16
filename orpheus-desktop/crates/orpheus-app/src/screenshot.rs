//! Screenshot capture functionality
//!
//! Provides screenshot capture for visual testing and debugging.
//! Uses egui's ViewportCommand::Screenshot to capture the current frame.

use chrono::Local;
use egui::{ColorImage, Context, ViewportCommand, ViewportId};
use image::{ImageBuffer, Rgba};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};

/// Screenshot manager for capturing and saving frames
#[derive(Default)]
pub struct ScreenshotManager {
    /// Whether a screenshot request is pending
    pending_request: bool,
    /// Path to save the next screenshot (if specified)
    pending_path: Option<PathBuf>,
    /// Callback for test harness
    pending_callback: Option<Arc<Mutex<Option<ColorImage>>>>,
    /// Screenshots directory
    screenshots_dir: PathBuf,
}

impl ScreenshotManager {
    pub fn new() -> Self {
        let screenshots_dir = dirs::picture_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Orpheus")
            .join("Screenshots");

        Self {
            pending_request: false,
            pending_path: None,
            pending_callback: None,
            screenshots_dir,
        }
    }

    /// Set custom screenshots directory
    pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.screenshots_dir = dir.into();
        self
    }

    /// Request a screenshot to be saved to the default directory
    pub fn request_screenshot(&mut self, ctx: &Context) {
        self.pending_request = true;
        self.pending_path = None;
        ctx.send_viewport_cmd(ViewportCommand::Screenshot);
        debug!("Screenshot requested");
    }

    /// Request a screenshot to be saved to a specific path
    pub fn request_screenshot_to(&mut self, ctx: &Context, path: impl Into<PathBuf>) {
        self.pending_request = true;
        self.pending_path = Some(path.into());
        ctx.send_viewport_cmd(ViewportCommand::Screenshot);
        debug!("Screenshot requested to specific path");
    }

    /// Request a screenshot for testing (returns via callback)
    pub fn request_screenshot_callback(
        &mut self,
        ctx: &Context,
        callback: Arc<Mutex<Option<ColorImage>>>,
    ) {
        self.pending_request = true;
        self.pending_callback = Some(callback);
        ctx.send_viewport_cmd(ViewportCommand::Screenshot);
        debug!("Screenshot requested with callback");
    }

    /// Check for completed screenshots in the event queue
    /// Call this in your update() method
    pub fn handle_events(&mut self, ctx: &Context) {
        if !self.pending_request {
            return;
        }

        ctx.input(|input| {
            for event in &input.events {
                if let egui::Event::Screenshot { viewport_id: _, image } = event {
                    self.handle_screenshot_result(image.clone());
                }
            }
        });
    }

    /// Handle a completed screenshot
    fn handle_screenshot_result(&mut self, image: Arc<ColorImage>) {
        self.pending_request = false;

        // If we have a callback (test harness), use that
        if let Some(callback) = self.pending_callback.take() {
            if let Ok(mut guard) = callback.lock() {
                *guard = Some((*image).clone());
            }
            debug!("Screenshot passed to callback");
            return;
        }

        // Otherwise save to file
        let path = self.pending_path.take().unwrap_or_else(|| {
            // Ensure directory exists
            if let Err(e) = std::fs::create_dir_all(&self.screenshots_dir) {
                warn!("Failed to create screenshots directory: {}", e);
            }

            // Generate filename with timestamp
            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            self.screenshots_dir.join(format!("orpheus_{}.png", timestamp))
        });

        // Save the screenshot
        if let Err(e) = save_color_image(&image, &path) {
            error!("Failed to save screenshot: {}", e);
        } else {
            info!("Screenshot saved to: {}", path.display());
        }
    }

    /// Get the screenshots directory
    pub fn screenshots_dir(&self) -> &Path {
        &self.screenshots_dir
    }

    /// Check if a screenshot is pending
    pub fn is_pending(&self) -> bool {
        self.pending_request
    }
}

/// Save a ColorImage to a PNG file
fn save_color_image(image: &ColorImage, path: &Path) -> Result<(), image::ImageError> {
    let width = image.width() as u32;
    let height = image.height() as u32;

    // Convert egui ColorImage to image buffer
    let mut buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    for (x, y, pixel) in buffer.enumerate_pixels_mut() {
        let idx = (y * width + x) as usize;
        let color = image.pixels[idx];
        *pixel = Rgba([color.r(), color.g(), color.b(), color.a()]);
    }

    buffer.save(path)
}

/// Compare two images for similarity (returns percentage match)
pub fn compare_images(a: &ColorImage, b: &ColorImage) -> f32 {
    if a.width() != b.width() || a.height() != b.height() {
        return 0.0;
    }

    let total = a.pixels.len();
    if total == 0 {
        return 1.0;
    }

    let mut matching = 0;
    for (pa, pb) in a.pixels.iter().zip(b.pixels.iter()) {
        // Compare with some tolerance
        if color_distance(*pa, *pb) < 10 {
            matching += 1;
        }
    }

    matching as f32 / total as f32
}

/// Calculate color distance (Manhattan distance in RGB space)
fn color_distance(a: egui::Color32, b: egui::Color32) -> i32 {
    (a.r() as i32 - b.r() as i32).abs()
        + (a.g() as i32 - b.g() as i32).abs()
        + (a.b() as i32 - b.b() as i32).abs()
}

/// Load a reference screenshot for comparison
pub fn load_reference_image(path: &Path) -> Result<ColorImage, image::ImageError> {
    let img = image::open(path)?.to_rgba8();
    let (width, height) = img.dimensions();

    let pixels: Vec<egui::Color32> = img
        .pixels()
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();

    Ok(ColorImage {
        size: [width as usize, height as usize],
        pixels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screenshot_manager_creation() {
        let manager = ScreenshotManager::new();
        assert!(!manager.is_pending());
    }

    #[test]
    fn test_custom_output_dir() {
        let manager = ScreenshotManager::new()
            .with_output_dir("/tmp/test_screenshots");
        assert_eq!(manager.screenshots_dir(), Path::new("/tmp/test_screenshots"));
    }

    #[test]
    fn test_color_distance() {
        let a = egui::Color32::from_rgb(100, 100, 100);
        let b = egui::Color32::from_rgb(105, 100, 100);
        assert_eq!(color_distance(a, b), 5);
    }

    #[test]
    fn test_compare_identical_images() {
        let pixels = vec![egui::Color32::RED; 100];
        let a = ColorImage { size: [10, 10], pixels: pixels.clone() };
        let b = ColorImage { size: [10, 10], pixels };
        assert_eq!(compare_images(&a, &b), 1.0);
    }
}
