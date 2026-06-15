// In-headset UI screens. v1 is a single panel that navigates between screens
// (like OyasumiVR's quick menu): the home screen with the Sleep Mode toggle,
// and per-feature screens filled in across milestones.
pub mod quickmenu;

pub use quickmenu::{build_countdown, build_menu, build_toast, MenuAction, Screen};
