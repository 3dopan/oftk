use eframe::egui;
use std::collections::HashSet;
use std::path::PathBuf;
use crate::data::models::FileAlias;
use crate::data::models::DirectoryEntry;

/// ファイルツリー表示コンポーネント
pub struct FileTreeView {
    /// 展開されているノードのIDセット
    expanded_nodes: HashSet<String>,

    /// アイテムの高さ（px）
    item_height: f32,
}

impl Default for FileTreeView {
    fn default() -> Self {
        Self::new()
    }
}

impl FileTreeView {
    /// デフォルトのアイテム高さ
    const DEFAULT_ITEM_HEIGHT: f32 = 24.0;

    /// 新しい FileTreeView を作成
    pub fn new() -> Self {
        Self {
            expanded_nodes: HashSet::new(),
            item_height: Self::DEFAULT_ITEM_HEIGHT,
        }
    }

    /// ツリーを描画（仮想化対応）
    ///
    /// # 戻り値
    /// (シングルクリックで選択されたインデックス, ダブルクリックで開くインデックス)
    ///
    /// # パフォーマンス最適化
    /// - 大量のアイテムでもスムーズに表示するため、仮想化を実装
    /// - ScrollAreaを使用して表示範囲のみをレンダリング
    /// - お気に入りアイテムを上部に表示（ソート済み）
    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        items: &[FileAlias],
        selected_index: Option<usize>,
    ) -> (Option<usize>, Option<usize>) {
        let mut selected_result = None;
        let mut open_result = None;

        // お気に入りを上部に表示するためにソート
        let mut sorted_items: Vec<(usize, &FileAlias)> = items.iter().enumerate().collect();
        sorted_items.sort_by(|a, b| {
            // お気に入りを優先（is_favoriteがtrueのものを先に）
            match (b.1.is_favorite, a.1.is_favorite) {
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                _ => std::cmp::Ordering::Equal,
            }
        });

        // アイテム数が少ない場合は通常のレンダリング
        if items.len() < 100 {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (original_index, item) in sorted_items.iter() {
                        let (selected, open) = self.render_item(ui, item, *original_index, selected_index);
                        if selected.is_some() {
                            selected_result = selected;
                        }
                        if open.is_some() {
                            open_result = open;
                        }
                    }
                });
        } else {
            // アイテム数が多い場合は仮想化レンダリング
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_rows(
                    ui,
                    self.item_height,
                    sorted_items.len(),
                    |ui, row_range| {
                        for index in row_range {
                            if index < sorted_items.len() {
                                let (original_index, item) = sorted_items[index];
                                let (selected, open) = self.render_item(ui, item, original_index, selected_index);
                                if selected.is_some() {
                                    selected_result = selected;
                                }
                                if open.is_some() {
                                    open_result = open;
                                }
                            }
                        }
                    },
                );
        }

        (selected_result, open_result)
    }

    /// 個別のアイテムを描画（再帰的）
    ///
    /// # 戻り値
    /// (シングルクリックで選択されたインデックス, ダブルクリックで開くインデックス)
    fn render_item(
        &mut self,
        ui: &mut egui::Ui,
        item: &FileAlias,
        index: usize,
        selected_index: Option<usize>,
    ) -> (Option<usize>, Option<usize>) {
        let is_expanded = self.is_expanded(&item.id);
        let is_folder = item.path.is_dir();
        let is_selected = selected_index == Some(index);
        let mut selected = None;
        let mut open = None;

        ui.horizontal(|ui| {
            // 展開/折りたたみアイコン（フォルダのみ）
            if is_folder {
                let icon = if is_expanded { "▼" } else { "▶" };
                if ui.button(icon).clicked() {
                    self.toggle_expansion(&item.id);
                }
            } else {
                ui.add_space(20.0);
            }

            // アイコン
            let icon = self.get_icon(item);
            ui.label(icon);

            // エイリアス名（選択可能）
            let response = ui.selectable_label(is_selected, &item.alias);

            // シングルクリック → 選択のみ
            if response.clicked() {
                selected = Some(index);
            }

            // ダブルクリック → 開く
            if response.double_clicked() {
                open = Some(index);
            }

            // パス
            ui.label(format!("-> {}", item.path.display()));
        });

        (selected, open)
    }

    /// ノードの展開状態をトグル
    fn toggle_expansion(&mut self, id: &str) {
        if self.expanded_nodes.contains(id) {
            self.expanded_nodes.remove(id);
        } else {
            self.expanded_nodes.insert(id.to_string());
        }
    }

    /// ノードが展開されているかチェック
    fn is_expanded(&self, id: &str) -> bool {
        self.expanded_nodes.contains(id)
    }

    /// アイテムのアイコンを取得
    fn get_icon(&self, item: &FileAlias) -> &'static str {
        // お気に入りの場合
        if item.is_favorite {
            return "⭐";
        }

        // フォルダの場合
        if item.path.is_dir() {
            return "📁";
        }

        // ファイルの場合
        "📄"
    }

    /// すべてのノードを展開
    pub fn expand_all(&mut self, items: &[FileAlias]) {
        for item in items {
            self.expanded_nodes.insert(item.id.clone());
        }
    }

    /// すべてのノードを折りたたみ
    pub fn collapse_all(&mut self) {
        self.expanded_nodes.clear();
    }

    /// DirectoryEntryのリストをレンダリング
    pub fn render_directory_entries(
        &mut self,
        ui: &mut egui::Ui,
        entries: &[DirectoryEntry],
        selected_index: Option<usize>,
    ) -> Option<usize> {
        // エントリをディレクトリ優先でソート
        let mut sorted_entries: Vec<(usize, &DirectoryEntry)> = entries
            .iter()
            .enumerate()
            .collect();

        sorted_entries.sort_by(|(_, a), (_, b)| {
            // ディレクトリを優先
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        let mut clicked_index = None;

        // 仮想化レンダリング（100件以上の場合）
        if sorted_entries.len() >= 100 {
            egui::ScrollArea::vertical().show_rows(
                ui,
                self.item_height,
                sorted_entries.len(),
                |ui, row_range| {
                    for row in row_range {
                        if let Some((original_idx, entry)) = sorted_entries.get(row) {
                            let is_selected = selected_index == Some(*original_idx);
                            if self.render_directory_entry_row(ui, entry, is_selected) {
                                clicked_index = Some(*original_idx);
                            }
                        }
                    }
                },
            );
        } else {
            for (original_idx, entry) in &sorted_entries {
                let is_selected = selected_index == Some(*original_idx);
                if self.render_directory_entry_row(ui, entry, is_selected) {
                    clicked_index = Some(*original_idx);
                }
            }
        }

        clicked_index
    }

    /// DirectoryEntry単体の行をレンダリング
    fn render_directory_entry_row(&self, ui: &mut egui::Ui, entry: &DirectoryEntry, is_selected: bool) -> bool {
        let icon = if entry.is_directory {
            if entry.is_wsl_path() {
                "🐧"  // WSLディレクトリ
            } else {
                "📁"  // 通常のディレクトリ
            }
        } else {
            "📄"
        };
        let label = format!("{} {}", icon, entry.name);
        ui.selectable_label(is_selected, label).clicked()
    }

    /// 単一のディレクトリノードを再帰的にレンダリング
    ///
    /// # 引数
    /// - `ui`: egui UI コンテキスト
    /// - `entry`: レンダリングするディレクトリエントリ
    /// - `flat_index`: グローバルフラットインデックスのアキュムレータ
    /// - `expanded_dirs`: 展開されているディレクトリのセット
    /// - `selected_index`: 選択されているインデックス
    /// - `level`: 階層レベル（0 = ルート）
    ///
    /// # 戻り値
    /// (シングルクリックで選択されたパス, ダブルクリックで開くパス, 右クリックかどうか)
    fn render_tree_node(
        &mut self,
        ui: &mut egui::Ui,
        entry: &DirectoryEntry,
        flat_index: &mut usize,
        expanded_dirs: &mut HashSet<PathBuf>,
        selected_index: Option<usize>,
        level: usize,
        pasted_highlight: Option<&crate::app::state::PastedFileHighlight>,
    ) -> (Option<PathBuf>, Option<PathBuf>, bool) {
        // ディレクトリのみ処理
        if !entry.is_directory {
            return (None, None, false);
        }

        // 現在のアイテムのインデックスを取得
        let current_index = *flat_index;
        *flat_index += 1;  // 次のアイテムのためにインクリメント

        let is_expanded = expanded_dirs.contains(&entry.path);
        let is_selected = selected_index == Some(current_index);
        let icon = if is_expanded { "▼" } else { "▶" };
        let mut selected_result: Option<PathBuf> = None;
        let mut open_result: Option<PathBuf> = None;
        let mut is_right_click = false;

        // ペースト直後のハイライト判定
        let is_pasted = pasted_highlight
            .map(|h| h.contains(&entry.path))
            .unwrap_or(false);

        ui.horizontal(|ui| {
            // 階層レベルに応じたインデント
            ui.add_space(level as f32 * 20.0);

            // 展開/折りたたみボタン
            if ui.small_button(icon).clicked() {
                if is_expanded {
                    expanded_dirs.remove(&entry.path);
                } else {
                    expanded_dirs.insert(entry.path.clone());
                }
            }

            // フォルダアイコンと名前
            let folder_icon = if entry.is_wsl_path() { "🐧" } else { "📁" };
            let label = format!("{} {}", folder_icon, entry.name);

            let response = if is_pasted && !is_selected {
                // ペースト直後: 緑背景（事前に設定）
                ui.scope(|ui| {
                    // 背景色を設定
                    ui.visuals_mut().widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(200, 255, 200);
                    ui.selectable_label(is_selected, label)
                }).inner
            } else if is_pasted && is_selected {
                // 選択中かつペースト直後: 青背景 + 緑枠線
                ui.scope(|ui| {
                    // 選択状態の背景 + 緑枠線
                    ui.visuals_mut().selection.stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 100));
                    ui.selectable_label(is_selected, label)
                }).inner
            } else {
                // 通常
                ui.selectable_label(is_selected, label)
            };

            // シングルクリック → 選択のみ
            if response.clicked() {
                selected_result = Some(entry.path.clone());
            }
            // ダブルクリック → 開く
            if response.double_clicked() {
                open_result = Some(entry.path.clone());
            }
            // 右クリック
            if response.secondary_clicked() {
                selected_result = Some(entry.path.clone());
                is_right_click = true;
            }
        });

        // 展開されている場合、サブアイテムを再帰的に表示
        if is_expanded {
            ui.indent(format!("indent_{}", entry.path.display()), |ui| {
                if let Ok(sub_entries) = std::fs::read_dir(&entry.path) {
                    let mut sub_items: Vec<DirectoryEntry> = sub_entries
                        .filter_map(|e| e.ok())
                        .filter_map(|e| DirectoryEntry::from_path(e.path()).ok())
                        .collect();

                    // ディレクトリ優先でソート
                    sub_items.sort_by(|a, b| {
                        match (a.is_directory, b.is_directory) {
                            (true, false) => std::cmp::Ordering::Less,
                            (false, true) => std::cmp::Ordering::Greater,
                            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                        }
                    });

                    // サブアイテムを処理
                    for sub_entry in sub_items.iter() {
                        if sub_entry.is_directory {
                            // ディレクトリは再帰的に処理
                            let (sub_selected, sub_open, sub_right_click) = self.render_tree_node(
                                ui,
                                sub_entry,
                                flat_index,  // アキュムレータを渡す（インクリメントされ続ける）
                                expanded_dirs,
                                selected_index,  // 選択状態を渡す
                                level + 1,  // 階層レベルを1つ増やす
                                pasted_highlight,  // ハイライト情報を渡す
                            );

                            if sub_selected.is_some() {
                                selected_result = sub_selected;
                                is_right_click = sub_right_click;
                            }
                            if sub_open.is_some() {
                                open_result = sub_open;
                            }
                        } else {
                            // ファイルはシンプルに表示
                            ui.horizontal(|ui| {
                                ui.add_space((level + 1) as f32 * 20.0);
                                let response = ui.label(format!("📄 {}", sub_entry.name));

                                // シングルクリック → 選択のみ
                                if response.clicked() {
                                    selected_result = Some(sub_entry.path.clone());
                                }
                                // ダブルクリック → 開く
                                if response.double_clicked() {
                                    open_result = Some(sub_entry.path.clone());
                                }
                                // 右クリック
                                if response.secondary_clicked() {
                                    selected_result = Some(sub_entry.path.clone());
                                    is_right_click = true;
                                }
                            });
                        }
                    }
                }
            });
        }

        (selected_result, open_result, is_right_click)
    }

    /// DirectoryEntryをツリー形式でレンダリング（エントリーポイント）
    ///
    /// # 引数
    /// - `ui`: egui UI コンテキスト
    /// - `entries`: レンダリングするエントリのリスト
    /// - `expanded_dirs`: 展開されているディレクトリのセット
    /// - `selected_index`: 選択されているインデックス
    ///
    /// # 戻り値
    /// (シングルクリックで選択されたパス, ダブルクリックで開くパス, 右クリックかどうか, 総アイテム数)
    pub fn render_directory_tree(
        &mut self,
        ui: &mut egui::Ui,
        entries: &[DirectoryEntry],
        expanded_dirs: &mut HashSet<PathBuf>,
        selected_index: Option<usize>,
        pasted_highlight: Option<&crate::app::state::PastedFileHighlight>,
    ) -> (Option<PathBuf>, Option<PathBuf>, bool, usize) {
        let mut selected_result: Option<PathBuf> = None;
        let mut open_result: Option<PathBuf> = None;
        let mut is_right_click = false;
        let mut flat_index = 0;  // アキュムレータを初期化

        for entry in entries.iter() {
            let is_selected = selected_index == Some(flat_index);

            if entry.is_directory {
                // ディレクトリは render_tree_node() に委譲
                let (sub_selected, sub_open, sub_right_click) = self.render_tree_node(
                    ui,
                    entry,
                    &mut flat_index,  // アキュムレータを渡す
                    expanded_dirs,
                    selected_index,
                    0,  // ルートレベル（階層 = 0）
                    pasted_highlight,  // ハイライト情報を渡す
                );

                if sub_selected.is_some() {
                    selected_result = sub_selected;
                    is_right_click = sub_right_click;
                }
                if sub_open.is_some() {
                    open_result = sub_open;
                }
            } else {
                // ファイルは従来通りの処理
                ui.horizontal(|ui| {
                    let label = format!("📄 {}", entry.name);
                    let response = ui.selectable_label(is_selected, label);

                    // シングルクリック → 選択のみ
                    if response.clicked() {
                        selected_result = Some(entry.path.clone());
                    }
                    // ダブルクリック → 開く
                    if response.double_clicked() {
                        open_result = Some(entry.path.clone());
                    }
                    // 右クリック
                    if response.secondary_clicked() {
                        selected_result = Some(entry.path.clone());
                        is_right_click = true;
                    }
                });
                flat_index += 1;  // ファイルもカウント
            }
        }

        (selected_result, open_result, is_right_click, flat_index)  // 総アイテム数を返す
    }
}
