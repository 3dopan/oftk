use eframe::egui;

/// テーマ（ライト/ダーク）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}

impl Theme {
    /// egui::Visuals に変換
    pub fn to_visuals(&self) -> egui::Visuals {
        match self {
            Theme::Light => egui::Visuals::light(),
            Theme::Dark => egui::Visuals::dark(),
        }
    }

    /// 文字列から Theme に変換
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            _ => None,
        }
    }

    /// Theme を文字列に変換
    pub fn to_string(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}
