use crate::data::models::FileHistory;

/// 履歴表示UI
pub struct HistoryView;

impl HistoryView {
    /// 新しい HistoryView を作成
    pub fn new() -> Self {
        Self
    }

    /// 履歴を表示
    pub fn render(&self, ui: &mut egui::Ui, history: &[FileHistory]) {
        ui.heading("最近開いたファイル");

        ui.separator();

        if history.is_empty() {
            ui.label("履歴はありません");
            return;
        }

        // 最大10件表示
        for entry in history.iter().take(10) {
            ui.horizontal(|ui| {
                // パス表示
                let path_str = entry.path.display().to_string();
                ui.label(&path_str);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // アクセス回数
                    ui.label(format!("{}回", entry.access_count));

                    // アクセス日時
                    let datetime_str = entry.accessed_at.format("%Y-%m-%d %H:%M").to_string();
                    ui.label(datetime_str);
                });
            });

            ui.separator();
        }
    }

    /// 履歴をクリックできる形式で表示（パスを返す）
    pub fn render_interactive(&self, ui: &mut egui::Ui, history: &[FileHistory]) -> Option<std::path::PathBuf> {
        ui.heading("最近開いたファイル");

        ui.separator();

        if history.is_empty() {
            ui.label("履歴はありません");
            return None;
        }

        let mut selected_path = None;

        // 最大10件表示
        for entry in history.iter().take(10) {
            ui.horizontal(|ui| {
                // パス表示（クリック可能なボタンとして）
                let path_str = entry.path.display().to_string();
                if ui.button(&path_str).clicked() {
                    selected_path = Some(entry.path.clone());
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // アクセス回数
                    ui.label(format!("{}回", entry.access_count));

                    // アクセス日時
                    let datetime_str = entry.accessed_at.format("%Y-%m-%d %H:%M").to_string();
                    ui.label(datetime_str);
                });
            });

            ui.separator();
        }

        selected_path
    }
}

impl Default for HistoryView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use chrono::Utc;

    #[test]
    fn test_history_view_creation() {
        let view = HistoryView::new();
        // 単純に作成できることを確認
        assert_eq!(std::mem::size_of_val(&view), 0);
    }

    #[test]
    fn test_history_view_default() {
        let view = HistoryView::default();
        // デフォルトで作成できることを確認
        assert_eq!(std::mem::size_of_val(&view), 0);
    }

    #[test]
    fn test_history_data_structure() {
        let now = Utc::now();
        let history = vec![
            FileHistory {
                path: PathBuf::from("/path/to/file1"),
                accessed_at: now,
                access_count: 5,
            },
            FileHistory {
                path: PathBuf::from("/path/to/file2"),
                accessed_at: now,
                access_count: 3,
            },
        ];

        assert_eq!(history.len(), 2);
        assert_eq!(history[0].path, PathBuf::from("/path/to/file1"));
        assert_eq!(history[0].access_count, 5);
        assert_eq!(history[1].path, PathBuf::from("/path/to/file2"));
        assert_eq!(history[1].access_count, 3);
    }

    #[test]
    fn test_history_empty_list() {
        let history: Vec<FileHistory> = vec![];
        assert_eq!(history.len(), 0);
        assert!(history.is_empty());
    }

    #[test]
    fn test_history_take_limit() {
        let now = Utc::now();
        let mut history = vec![];

        // 15個のエントリを作成
        for i in 1..=15 {
            history.push(FileHistory {
                path: PathBuf::from(format!("/path/to/file{}", i)),
                accessed_at: now,
                access_count: i as u32,
            });
        }

        // 最大10件まで取得
        let limited: Vec<_> = history.iter().take(10).collect();
        assert_eq!(limited.len(), 10);
    }
}
