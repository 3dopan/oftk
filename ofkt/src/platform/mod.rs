pub mod theme_detector;
pub mod system_tray;
pub mod hotkey;
pub mod edge_detector;
pub mod autostart;
pub mod drives;

// Re-export for convenience
pub use system_tray::{SystemTray, TrayEvent};
pub use theme_detector::detect_system_theme;
pub use hotkey::HotkeyManager;
pub use edge_detector::EdgeDetector;
pub use autostart::AutostartManager;
pub use drives::{DriveInfo, DriveType, get_drives, get_wsl_distributions, get_quick_access};
