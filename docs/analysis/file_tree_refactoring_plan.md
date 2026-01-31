# file_tree.rs リファクタリング計画書

## 1. 現状分析

### 1.1 現在のrender_directory_tree()の構造（259-358行目）

現在の実装は**2階層まで**しか対応していません。

#### 1階層目（ルートレベル）の処理（269-300行目）
```rust
for (idx, entry) in entries.iter().enumerate() {
    if entry.is_directory {
        // 展開/折りたたみボタンあり
        if ui.small_button(icon).clicked() { ... }

        // クリック検出あり
        if response.clicked() { clicked_index = Some(idx); }
        if response.secondary_clicked() { ... }
    }
}
```

**特徴**:
- 展開/折りたたみボタン: あり
- クリック検出: あり
- 選択状態のハイライト: あり
- 展開状態の管理: `expanded_dirs` HashSetで管理

#### 2階層目（サブディレクトリ）の処理（302-338行目）
```rust
if is_expanded {
    ui.indent(..., |ui| {
        // サブアイテムを再帰的に表示（シンプル版）
        for sub_entry in &sub_items {
            ui.horizontal(|ui| {
                ui.label(format!("  {} {}", sub_icon, sub_entry.name));
            });
        }
    });
}
```

**特徴**:
- 展開/折りたたみボタン: **なし**
- クリック検出: **なし**
- 選択状態のハイライト: **なし**
- 単純なラベル表示のみ

### 1.2 問題点

1. **2階層目以降は静的表示のみ**
   - サブディレクトリをクリックしても反応しない
   - 3階層以上のディレクトリは表示されない

2. **処理の重複**
   - 1階層目と2階層目で同じUI要素（アイコン、名前表示）を別々に実装
   - DRY原則違反

3. **拡張性の欠如**
   - 3階層以上に対応するには根本的な設計変更が必要
   - 現在のコード構造では再帰的な処理が不可能

## 2. 設計変更の方針

### 2.1 再帰的アーキテクチャへの移行

現在の**フラットな2階層処理**から**再帰的な多階層処理**に設計変更します。

#### 変更前（現在）
```
render_directory_tree()
├─ 1階層目: 完全な処理
└─ 2階層目: シンプル表示のみ（再帰なし）
```

#### 変更後（目標）
```
render_directory_tree()  ← エントリーポイント
└─ render_tree_node()    ← 再帰的ヘルパー
   └─ render_tree_node() ← 自己呼び出し（無限階層対応）
      └─ render_tree_node() ...
```

### 2.2 メソッド分割

#### 公開メソッド: `render_directory_tree()`
- **役割**: エントリーポイント
- **処理**: ルートエントリをループし、各エントリを`render_tree_node()`に委譲
- **シグネチャ**: 変更なし（既存の呼び出し元に影響を与えない）

#### プライベートヘルパー: `render_tree_node()`
- **役割**: 単一のディレクトリノードを再帰的に描画
- **処理**:
  1. ディレクトリのUI要素を描画
  2. 展開されている場合、サブディレクトリを読み込み
  3. 各サブディレクトリに対して**自身を再帰的に呼び出す**
- **引数に追加**: `level: usize`（階層レベル）

### 2.3 インデント処理

階層レベル(`level`)に応じてインデントを動的に計算します。

```rust
ui.add_space(level as f32 * 20.0);  // 階層ごとに20px
```

- level 0（ルート）: 0px
- level 1（1階層目）: 20px
- level 2（2階層目）: 40px
- level 3（3階層目）: 60px
- ...

## 3. 実装計画

### 3.1 新規メソッド: `render_tree_node()`

```rust
fn render_tree_node(
    &mut self,
    ui: &mut egui::Ui,
    entry: &DirectoryEntry,
    index: usize,
    expanded_dirs: &mut HashSet<PathBuf>,
    selected_index: Option<usize>,
    level: usize,  // 階層レベル（0 = ルート）
) -> (Option<usize>, bool) {
    // ディレクトリのみ処理
    if !entry.is_directory {
        return (None, false);
    }

    // 1. 展開状態の取得
    let is_expanded = expanded_dirs.contains(&entry.path);

    // 2. UI要素の描画
    ui.horizontal(|ui| {
        // インデント
        ui.add_space(level as f32 * 20.0);

        // 展開/折りたたみボタン
        // フォルダアイコンと名前
        // クリック検出
    });

    // 3. 展開されている場合、サブディレクトリを再帰的に処理
    if is_expanded {
        ui.indent(..., |ui| {
            for sub_entry in &sub_items {
                if sub_entry.is_directory {
                    // ★ 再帰呼び出し
                    self.render_tree_node(
                        ui, sub_entry, sub_idx,
                        expanded_dirs, None,
                        level + 1  // 階層レベルを1つ増やす
                    );
                } else {
                    // ファイルはシンプル表示
                }
            }
        });
    }

    (clicked_index, is_right_click)
}
```

### 3.2 既存メソッドのリファクタリング: `render_directory_tree()`

```rust
pub fn render_directory_tree(
    &mut self,
    ui: &mut egui::Ui,
    entries: &[DirectoryEntry],
    expanded_dirs: &mut HashSet<PathBuf>,
    selected_index: Option<usize>,
) -> (Option<usize>, bool) {
    let mut clicked_result = (None, false);

    for (idx, entry) in entries.iter().enumerate() {
        if entry.is_directory {
            // ディレクトリは render_tree_node() に委譲
            let result = self.render_tree_node(
                ui, entry, idx,
                expanded_dirs, selected_index,
                0  // ルートレベル
            );

            if result.0.is_some() {
                clicked_result = result;
            }
        } else {
            // ファイルは従来通りの処理
            // (339-354行目のロジックを維持)
        }
    }

    clicked_result
}
```

## 4. インデックス管理の問題点

### 4.1 現在の問題

現在の`selected_index: Option<usize>`は**フラットなインデックス**です。

```rust
entries[0]  // ルートのディレクトリA
entries[1]  // ルートのディレクトリB
entries[2]  // ルートのファイルC
```

しかし、階層構造では：

```
entries[0]  // ディレクトリA
  └─ sub_items[0]  // サブディレクトリA-1
  └─ sub_items[1]  // サブディレクトリA-2
entries[1]  // ディレクトリB
```

サブアイテムのインデックス`sub_items[0]`は、親の`entries`配列とは無関係です。

### 4.2 短期的な対処

現在の実装では、**サブアイテムの選択状態は無視**します。

```rust
self.render_tree_node(
    ui, sub_entry, sub_idx,
    expanded_dirs,
    None,  // サブアイテムの選択状態は別管理が必要
    level + 1
);
```

### 4.3 長期的な改善案（Task #72で検討）

`selected_index`を`selected_path: Option<PathBuf>`に変更し、パスベースで選択状態を管理します。

**メリット**:
- 階層に依存しない選択状態管理
- サブディレクトリの選択にも対応可能

**デメリット**:
- 既存コードへの影響範囲が大きい
- app/state.rs, app/mod.rs の変更が必要

## 5. パフォーマンスの考慮

### 5.1 仮想化レンダリングとの併用

現在、`render_directory_entries()`（206-233行目）では100件以上の場合に仮想化レンダリングを使用しています。

再帰的ツリー表示では：
- **仮想化は困難**（階層が動的に変化するため）
- 代わりに**遅延読み込み**を活用
  - 展開されたディレクトリのみ読み込む
  - 折りたたまれたディレクトリは読み込まない

### 5.2 大規模ディレクトリ対策

- `std::fs::read_dir()`の結果を即座にVecに変換
- ソートは必要最小限に抑える
- 不要な`clone()`を避ける

## 6. テスト計画

### 6.1 機能テスト（Task #70）
1. 3階層以上のディレクトリを作成
2. 各階層で展開/折りたたみが動作することを確認
3. インデントが正しく表示されることを確認
4. クリックイベントが正しく処理されることを確認

### 6.2 パフォーマンステスト（Task #71）
1. 大規模ディレクトリ（C:\Windows, C:\Program Files）での動作確認
2. 複数階層の同時展開時の応答性確認
3. メモリ使用量の監視

## 7. リスク分析

### 7.1 高リスク項目
- **インデックス管理の複雑化**: サブアイテムのインデックスと親のインデックスが混在
  - 対策: 短期的にはサブアイテムの選択を無視、長期的にはパスベース選択に移行

### 7.2 中リスク項目
- **パフォーマンス劣化**: 深い階層の展開時に遅延が発生する可能性
  - 対策: 遅延読み込み、必要に応じてキャッシュ機構の導入

### 7.3 低リスク項目
- **既存の動作への影響**: 公開メソッドのシグネチャは変更しないため、呼び出し元への影響は最小限

## 8. まとめ

### 8.1 変更の要点
1. `render_tree_node()`プライベートヘルパーメソッドを新規追加
2. `render_directory_tree()`をエントリーポイントとして再設計
3. 再帰的な呼び出しで無限階層に対応
4. 階層レベル(`level`)に応じたインデント処理

### 8.2 実装の順序
1. Task #67: `render_tree_node()`の実装
2. Task #68: `render_directory_tree()`のリファクタリング
3. Task #69: ビルドテストと構文エラー修正
4. Task #70: 機能テスト（3階層以上）
5. Task #71: パフォーマンステスト
6. Task #72: 選択状態管理の改善検討（オプション）

### 8.3 成功基準
- 3階層以上のディレクトリが正しく表示される
- 全ての階層で展開/折りたたみが動作する
- インデントが階層ごとに正しく表示される
- 既存の動作が壊れない
- パフォーマンスが維持される

---

**作成日**: 2026-01-29
**作成者**: analysis-agent
**レビュー状況**: 未レビュー
