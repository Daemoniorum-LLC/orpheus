/// UI modules

mod transport;
mod modes;
mod tracks;
mod effects;
mod mixer;
mod compose;
mod master;
mod practice;
mod settings;

pub use transport::transport_panel;
pub use modes::mode_selector;
pub use tracks::track_list;
pub use effects::effects_panel;
pub use mixer::mixer_view;
pub use compose::compose_view;
pub use master::master_view;
pub use practice::practice_view;
pub use settings::settings_window;
