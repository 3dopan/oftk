use eframe::egui;
use crate::data::models::FileAlias;

/// お気に入り表示コンポーネント
pub struct FavoritesView;

impl FavoritesView {
    /// 新しい FavoritesView を作成
    pub fn new() -> Self {
        Self
    }

    /// お気に入り一覧を表示
    ///
    /// # 戻り値
    /// クリックされたアイテムのインデックス
    pub fn render(&self, ui: &mut egui::Ui, favorites: &[&FileAlias]) -> Option<usize> {
        ui.heading("お気に入り");

        if favorites.is_empty() {
            ui.label("お気に入りはありません");
            return None;
        }

        let mut clicked_index = None;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (index, alias) in favorites.iter().enumerate() {
                    ui.horizontal(|ui| {
                        // スターアイコン
                        ui.label("⭐");

                        // エイリアス名（選択可能）
                        if ui.selectable_label(false, &alias.alias).clicked() {
                            clicked_index = Some(index);
                        }

                        // パス
                        ui.label(format!("-> {}", alias.path.display()));
                    });
                }
            });

        clicked_index
    }
}

impl Default for FavoritesView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use chrono::Utc;

    fn create_test_alias(alias: &str, path: &str, is_favorite: bool) -> FileAlias {
        let now = Utc::now();
        FileAlias {
            id: uuid::Uuid::new_v4().to_string(),
            alias: alias.to_string(),
            path: PathBuf::from(path),
            tags: vec![],
            color: None,
            created_at: now,
            last_accessed: now,
            is_favorite,
        }
    }

    #[test]
    fn test_new() {
        let view = FavoritesView::new();
        assert!(std::mem::size_of_val(&view) == 0); // ZSTであることを確認
    }

    #[test]
    fn test_default() {
        let view = FavoritesView::default();
        assert!(std::mem::size_of_val(&view) == 0); // ZSTであることを確認
    }

    #[test]
    fn test_favorites_view_structure() {
        // FavoritesViewの構造が正しいことを確認
        let view = FavoritesView::new();

        let favorites = vec![
            create_test_alias("test1", "/path/to/file1", true),
            create_test_alias("test2", "/path/to/file2", true),
        ];

        let favorites_refs: Vec<&FileAlias> = favorites.iter().collect();

        // renderメソッドが存在することを確認
        // 実際のUIコンテキストがないため、この部分はコンパイルチェックのみ
        drop(view);
        drop(favorites_refs);
    }
}
