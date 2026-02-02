use eframe::egui;
use crate::data::models::DirectoryEntry;

/// コンテキストメニューで選択されたアクション
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuAction {
    /// ファイル/フォルダをデフォルトアプリケーションで開く
    Open,
    /// クリップボードまたは別の場所にコピー
    Copy,
    /// 切り取り（移動のため）
    Cut,
    /// クリップボードから貼り付け
    Paste,
    /// ファイル/フォルダを移動
    Move,
    /// 確認付きで削除
    Delete,
    /// ファイル/フォルダ名を変更
    Rename,
    /// プロパティを表示
    Properties,
    /// 選択したアイテムの新しいエイリアスを作成
    AddAlias,
}

/// コンテキストメニューコンポーネント
pub struct ContextMenu {
    // 必要に応じて状態を保持
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextMenu {
    /// 新しい ContextMenu を作成
    pub fn new() -> Self {
        Self {}
    }

    /// 右クリックされた時に呼ばれる
    ///
    /// このメソッドはUI要素のレスポンスに対して右クリックメニューを表示するために使用されます。
    /// 実際の使用例:
    /// ```ignore
    /// let response = ui.selectable_label(selected, "アイテム");
    /// if let Some(action) = context_menu.show(ui, item_index) {
    ///     // アクションを処理
    /// }
    /// ```
    ///
    /// # 引数
    /// * `ui` - egui の UI コンテキスト
    /// * `item_index` - 右クリックされたアイテムのインデックス（現在は使用されていませんが、将来の拡張のために保持）
    ///
    /// # 戻り値
    /// 選択されたアクション（あれば）
    pub fn show(&mut self, ui: &mut egui::Ui, _item_index: usize) -> Option<MenuAction> {
        let mut action = None;

        // このメソッドは通常、response.context_menu() と組み合わせて使用される
        // ここでは直接メニュー項目を表示
        ui.vertical(|ui| {
            action = Self::show_menu_items(ui);
        });

        action
    }

    /// メニュー項目を表示する内部ヘルパー関数
    ///
    /// # 引数
    /// * `ui` - egui の UI コンテキスト
    ///
    /// # 戻り値
    /// 選択されたアクション（あれば）
    fn show_menu_items(ui: &mut egui::Ui) -> Option<MenuAction> {
        let mut action = None;

        ui.set_min_width(180.0);

        // "開く" メニュー項目
        if ui.button("開く").clicked() {
            action = Some(MenuAction::Open);
            ui.close_menu();
        }

        ui.separator();

        // "コピー" メニュー項目
        if ui.button("コピー").clicked() {
            action = Some(MenuAction::Copy);
            ui.close_menu();
        }

        // "移動" メニュー項目
        if ui.button("移動").clicked() {
            action = Some(MenuAction::Move);
            ui.close_menu();
        }

        ui.separator();

        // "削除" メニュー項目
        if ui.button("削除").clicked() {
            action = Some(MenuAction::Delete);
            ui.close_menu();
        }

        // "名前変更" メニュー項目
        if ui.button("名前変更").clicked() {
            action = Some(MenuAction::Rename);
            ui.close_menu();
        }

        ui.separator();

        // "エイリアス追加" メニュー項目
        if ui.button("エイリアス追加").clicked() {
            action = Some(MenuAction::AddAlias);
            ui.close_menu();
        }

        action
    }

    /// レスポンスに対してコンテキストメニューを表示
    ///
    /// これが推奨される使用方法です。UI要素のレスポンスに対して右クリックメニューを表示します。
    ///
    /// 使用例:
    /// ```ignore
    /// let response = ui.selectable_label(selected, "アイテム");
    /// if let Some(action) = context_menu.show_for_response(&response, item_index) {
    ///     match action {
    ///         MenuAction::Open => { /* 開く処理 */ }
    ///         MenuAction::Delete => { /* 削除処理 */ }
    ///         // ...
    ///     }
    /// }
    /// ```
    ///
    /// # 引数
    /// * `response` - UI 要素のレスポンス
    /// * `_item_index` - アイテムのインデックス（将来の拡張のために保持）
    ///
    /// # 戻り値
    /// 選択されたアクション（あれば）
    pub fn show_for_response(
        &mut self,
        response: &egui::Response,
        _item_index: usize,
    ) -> Option<MenuAction> {
        let mut action = None;

        response.context_menu(|ui| {
            if let Some(a) = Self::show_menu_items(ui) {
                action = Some(a);
            }
        });

        action
    }

    /// DirectoryEntry用のコンテキストメニューを表示
    ///
    /// # 引数
    /// * `ui` - egui の UI コンテキスト
    /// * `_entry` - 右クリックされたDirectoryEntry（将来の拡張のために保持）
    ///
    /// # 戻り値
    /// 選択されたアクション（あれば）
    pub fn show_for_directory_entry(
        ui: &mut egui::Ui,
        _entry: &DirectoryEntry,
    ) -> Option<MenuAction> {
        let mut action = None;

        ui.set_min_width(180.0);

        // "開く" メニュー項目
        if ui.button("開く").clicked() {
            action = Some(MenuAction::Open);
            ui.close_menu();
        }

        ui.separator();

        // "コピー" メニュー項目
        if ui.button("コピー").clicked() {
            action = Some(MenuAction::Copy);
            ui.close_menu();
        }

        // "切り取り" メニュー項目
        if ui.button("切り取り").clicked() {
            action = Some(MenuAction::Cut);
            ui.close_menu();
        }

        // "削除" メニュー項目
        if ui.button("削除").clicked() {
            action = Some(MenuAction::Delete);
            ui.close_menu();
        }

        // "名前の変更" メニュー項目
        if ui.button("名前の変更").clicked() {
            action = Some(MenuAction::Rename);
            ui.close_menu();
        }

        ui.separator();

        // "プロパティ" メニュー項目
        if ui.button("プロパティ").clicked() {
            action = Some(MenuAction::Properties);
            ui.close_menu();
        }

        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_creation() {
        let _menu = ContextMenu::new();
    }

    #[test]
    fn test_menu_action_equality() {
        assert_eq!(MenuAction::Open, MenuAction::Open);
        assert_ne!(MenuAction::Open, MenuAction::Copy);
    }

    #[test]
    fn test_menu_action_debug() {
        let action = MenuAction::Open;
        let debug_str = format!("{:?}", action);
        assert_eq!(debug_str, "Open");
    }

    #[test]
    fn test_menu_action_clone() {
        let action = MenuAction::Copy;
        let cloned = action;
        assert_eq!(action, cloned);
    }

    #[test]
    fn test_all_menu_actions() {
        let actions = vec![
            MenuAction::Open,
            MenuAction::Copy,
            MenuAction::Cut,
            MenuAction::Paste,
            MenuAction::Move,
            MenuAction::Delete,
            MenuAction::Rename,
            MenuAction::Properties,
            MenuAction::AddAlias,
        ];

        // すべてのアクションが異なることを確認
        for (i, action1) in actions.iter().enumerate() {
            for (j, action2) in actions.iter().enumerate() {
                if i == j {
                    assert_eq!(action1, action2);
                } else {
                    assert_ne!(action1, action2);
                }
            }
        }
    }

    #[test]
    fn test_default_implementation() {
        let menu1 = ContextMenu::new();
        let menu2 = ContextMenu::default();
        // 両方のインスタンスが正常に作成されることを確認
        drop(menu1);
        drop(menu2);
    }
}
