# サイドバーキーボードナビゲーション実装レポート

## 実装日時
2026-01-29

## タスク概要
Task 5.4: サイドバーでの上下キー・Enterキー処理を実装

サイドバーにフォーカスがある時に、上下キーで項目を選択し、Enterキーで開けるようにする機能を実装しました。

## 実装内容

### 1. サイドバー項目の収集と追跡

サイドバーには以下の4つのセクションがあります：
- エイリアス（最大10件表示）
- クイックアクセス
- ドライブ
- WSL分布版

各セクションの項目を収集し、統一されたインデックス管理を実装しました。

**ファイル**: `ofkt/src/app/mod.rs`

#### エイリアスセクション（行411-440）
```rust
// エイリアスリストを表示（最大10件）
let displayed_aliases: Vec<_> = filtered_aliases.iter().take(10).collect();
let displayed_aliases_count = displayed_aliases.len();

for (alias_index, alias) in displayed_aliases.iter().enumerate() {
    let button_text = if alias.is_favorite {
        format!("⭐ {}", alias.alias)
    } else {
        alias.alias.clone()
    };

    let mut button = egui::Button::new(&button_text);
    if self.state.current_focus_area == FocusArea::Sidebar
        && self.state.selected_sidebar_index == Some(alias_index)
    {
        // 選択されている場合は黄色でハイライト
        button = button.fill(egui::Color32::from_rgb(60, 60, 30));
    }

    if ui.add(button).clicked() {
        // エイリアスのパスに移動
        if let Err(e) = self.state.init_directory_browser(alias.path.clone()) {
            log::error!("エイリアスパスへの移動に失敗: {}", e);
        } else {
            self.state.directory_search_query.clear();
            log::info!("エイリアス「{}」を開きました", alias.alias);
        }
    }
}
```

#### クイックアクセスセクション（行444-466）
```rust
// クイックアクセス
ui.label("クイックアクセス");
let quick_access = crate::platform::get_quick_access();
for (qa_index, drive) in quick_access.iter().enumerate() {
    let sidebar_index = displayed_aliases_count + qa_index;

    let mut button = egui::Button::new(&drive.name);
    if self.state.current_focus_area == FocusArea::Sidebar
        && self.state.selected_sidebar_index == Some(sidebar_index)
    {
        // 選択されている場合は黄色でハイライト
        button = button.fill(egui::Color32::from_rgb(60, 60, 30));
    }

    if ui.add(button).clicked() {
        if let Err(e) = self.state.init_directory_browser(drive.path.clone()) {
            log::error!("ディレクトリブラウザ初期化失敗: {}", e);
        } else {
            self.state.directory_search_query.clear();
        }
    }
}
```

#### ドライブセクション（行470-499）
```rust
// ドライブ
ui.label("ドライブ");
let drives = crate::platform::get_drives();
for (drive_index, drive) in drives.iter().enumerate() {
    let sidebar_index = displayed_aliases_count + quick_access.len() + drive_index;

    let icon = match drive.drive_type {
        crate::platform::DriveType::Fixed => "💿",
        crate::platform::DriveType::Removable => "💾",
        crate::platform::DriveType::Network => "🌐",
        _ => "📁",
    };

    let mut button = egui::Button::new(format!("{} {}", icon, drive.name));
    if self.state.current_focus_area == FocusArea::Sidebar
        && self.state.selected_sidebar_index == Some(sidebar_index)
    {
        // 選択されている場合は黄色でハイライト
        button = button.fill(egui::Color32::from_rgb(60, 60, 30));
    }

    if ui.add(button).clicked() {
        if let Err(e) = self.state.init_directory_browser(drive.path.clone()) {
            log::error!("ディレクトリブラウザ初期化失敗: {}", e);
        } else {
            self.state.directory_search_query.clear();
        }
    }
}
```

#### WSLセクション（行503-527）
```rust
// WSL
let wsl_dists = crate::platform::get_wsl_distributions();
if !wsl_dists.is_empty() {
    ui.label("WSL");
    for (wsl_index, dist) in wsl_dists.iter().enumerate() {
        let sidebar_index = displayed_aliases_count + quick_access.len() + drives.len() + wsl_index;

        let mut button = egui::Button::new(format!("🐧 {}", dist.name));
        if self.state.current_focus_area == FocusArea::Sidebar
            && self.state.selected_sidebar_index == Some(sidebar_index)
        {
            // 選択されている場合は黄色でハイライト
            button = button.fill(egui::Color32::from_rgb(60, 60, 30));
        }

        if ui.add(button).clicked() {
            if let Err(e) = self.state.init_directory_browser(dist.path.clone()) {
                log::error!("ディレクトリブラウザ初期化失敗: {}", e);
            } else {
                self.state.directory_search_query.clear();
            }
        }
    }
}
```

### 2. キーボードイベント処理（行530-614）

サイドバーにフォーカスがある場合のキーボード操作を実装しました：

```rust
// サイドバーにフォーカスがある場合のキー操作
if self.state.current_focus_area == FocusArea::Sidebar {
    // サイドバーの項目数を計算
    let sidebar_items_count =
        displayed_aliases_count  // エイリアスの数
        + quick_access.len()
        + drives.len()
        + wsl_dists.len();

    if sidebar_items_count > 0 {
        // 下キー: 次の項目を選択
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            let max_index = sidebar_items_count.saturating_sub(1);
            self.state.selected_sidebar_index = Some(
                self.state.selected_sidebar_index
                    .map(|i| (i + 1).min(max_index))
                    .unwrap_or(0)
            );
        }

        // 上キー: 前の項目を選択
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.state.selected_sidebar_index = self.state.selected_sidebar_index
                .and_then(|i| i.checked_sub(1));
        }

        // Enterキー: 選択された項目を開く
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Some(idx) = self.state.selected_sidebar_index {
                // インデックスに応じて対応する項目を開く
                let mut current_index = 0;

                // エイリアスセクション
                if idx < displayed_aliases_count {
                    if let Some(alias) = displayed_aliases.get(idx) {
                        if let Err(e) = self.state.init_directory_browser(alias.path.clone()) {
                            log::error!("エイリアスパスへの移動に失敗: {}", e);
                        } else {
                            self.state.directory_search_query.clear();
                        }
                    }
                } else {
                    current_index += displayed_aliases_count;

                    // クイックアクセスセクション
                    if idx < current_index + quick_access.len() {
                        let qa_idx = idx - current_index;
                        if let Some(drive) = quick_access.get(qa_idx) {
                            if let Err(e) = self.state.init_directory_browser(drive.path.clone()) {
                                log::error!("クイックアクセスへの移動に失敗: {}", e);
                            } else {
                                self.state.directory_search_query.clear();
                            }
                        }
                    } else {
                        current_index += quick_access.len();

                        // ドライブセクション
                        if idx < current_index + drives.len() {
                            let drive_idx = idx - current_index;
                            if let Some(drive) = drives.get(drive_idx) {
                                if let Err(e) = self.state.init_directory_browser(drive.path.clone()) {
                                    log::error!("ドライブへの移動に失敗: {}", e);
                                } else {
                                    self.state.directory_search_query.clear();
                                }
                            }
                        } else {
                            current_index += drives.len();

                            // WSLセクション
                            if idx < current_index + wsl_dists.len() {
                                let wsl_idx = idx - current_index;
                                if let Some(dist) = wsl_dists.get(wsl_idx) {
                                    if let Err(e) = self.state.init_directory_browser(dist.path.clone()) {
                                        log::error!("WSL分布版への移動に失敗: {}", e);
                                    } else {
                                        self.state.directory_search_query.clear();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

## 実装の特徴

### 1. 統一されたインデックス管理
サイドバー全体で統一されたインデックスを使用し、各セクションの項目を一貫した方法で管理しています：

- エイリアス: `0 ～ displayed_aliases_count - 1`
- クイックアクセス: `displayed_aliases_count ～ displayed_aliases_count + quick_access.len() - 1`
- ドライブ: `displayed_aliases_count + quick_access.len() ～ ...`
- WSL: `displayed_aliases_count + quick_access.len() + drives.len() ～ ...`

### 2. 視覚的フィードバック
選択された項目は黄色（RGB: 60, 60, 30）でハイライト表示されます。これにより、ユーザーは現在どの項目が選択されているかを一目で確認できます。

### 3. フォーカス管理との統合
既存のフォーカス管理システム（`FocusArea::Sidebar`）と統合されており、サイドバーにフォーカスがある場合のみキーボード操作が有効になります。

### 4. 境界チェック
- 下キー: `saturating_sub(1)`と`min(max_index)`を使用して範囲内に収める
- 上キー: `checked_sub(1)`を使用してアンダーフローを防止
- インデックス0未満では選択を解除（`None`）

## 使用方法

1. **ディレクトリモードに切り替え**: アプリケーションでディレクトリモードを選択
2. **サイドバーにフォーカス**: Tabキーを使ってサイドバーにフォーカスを移動
3. **項目を選択**: 上下キーでサイドバーの項目を選択（選択された項目は黄色でハイライト）
4. **項目を開く**: Enterキーで選択された項目を開く

## 検証方法

```bash
cd ofkt
cargo build
cargo run
```

実際にアプリを起動して：
1. Tabキーでサイドバーにフォーカス
2. 上下キーでサイドバーの項目を選択
3. 選択された項目が黄色でハイライト表示されることを確認
4. Enterキーで選択項目が開くことを確認

## 既存機能との連携

- **フォーカス管理**: Task 5.3で実装されたフォーカス管理システムと統合
- **検索機能**: 項目選択時に検索バーが自動的にクリアされる
- **エラーハンドリング**: 各操作でエラーが発生した場合、適切にログに記録

## 技術的な注意点

1. **変数スコープ**: `displayed_aliases`, `quick_access`, `drives`, `wsl_dists`は、サイドバーパネルの外側（キーボードイベント処理）でも使用されるため、適切なスコープで定義されています。

2. **ctx.input()の使用**: サイドバーパネル外でのキーイベント処理には`ctx.input()`を使用（`ui.input()`ではなく）。

3. **インデックス計算**: セクション間でのインデックス変換を正確に行うため、累積的な計算を使用しています。

## まとめ

サイドバーでのキーボードナビゲーション機能が完全に実装されました。ユーザーは、マウスを使わずにキーボードだけでサイドバーの項目を選択し、開くことができます。この機能は、既存のフォーカス管理システムとシームレスに統合されており、効率的なナビゲーションエクスペリエンスを提供します。

## 関連タスク

- Task 5.1: Tabキーによるフォーカス領域切替
- Task 5.2: 視覚的なフォーカスインジケーター
- Task 5.3: フォーカス領域管理システム実装
- **Task 5.4: サイドバーキーボードナビゲーション実装** ← 本実装

## ステータス

✅ 実装完了
