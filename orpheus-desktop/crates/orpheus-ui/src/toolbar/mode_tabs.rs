//! Mode selection tabs

use egui::{Color32, RichText, Ui};
use orpheus_core::ProductionMode;

/// Mode tab bar
pub struct ModeTabs;

impl ModeTabs {
    /// Show mode tabs, returns new mode if changed
    pub fn show(ui: &mut Ui, current: ProductionMode, accent: Color32) -> Option<ProductionMode> {
        let mut new_mode = None;

        ui.horizontal(|ui| {
            for mode in ProductionMode::all() {
                let is_selected = *mode == current;

                let text = RichText::new(format!("{} {}", mode.icon(), mode.display_name()));
                let text = if is_selected {
                    text.color(accent).strong()
                } else {
                    text.color(Color32::GRAY)
                };

                let response = ui.selectable_label(is_selected, text)
                    .on_hover_text(mode.description());

                if response.clicked() && !is_selected {
                    new_mode = Some(*mode);
                }
            }
        });

        new_mode
    }
}
