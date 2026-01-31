# ベンチマークセットアップガイド

## 実装内容

Task 6.1.1（パフォーマンス測定とプロファイリング）の実装が完了しました。

### 作成されたファイル

1. **src/lib.rs** - ライブラリクレート設定
   - ベンチマーク用にモジュールを公開
   - `SearchEngine`, `AliasManager`, `FileManager` を再エクスポート

2. **benches/search_benchmark.rs** - 検索処理のベンチマーク
   - 完全一致検索
   - 前方一致検索
   - ファジーマッチ検索
   - 階層パス検索
   - タグ検索
   - スケーラビリティテスト（10〜1000件）
   - キャッシュ効果の測定
   - 複雑なクエリテスト

3. **benches/alias_benchmark.rs** - エイリアス管理のベンチマーク
   - エイリアス追加（単一・複数・タグ付き）
   - エイリアス削除（IDまたは名前）
   - エイリアス更新（単一フィールド・全フィールド）
   - エイリアス取得
   - 重複チェック
   - 複雑な操作の組み合わせ

4. **benches/file_operations_benchmark.rs** - ファイル操作のベンチマーク
   - ファイルコピー（1KB, 100KB, 1MB）
   - コピーのスケーラビリティ
   - ファイル移動
   - ファイル削除（ファイル・ディレクトリ）
   - ファイル名変更
   - 複数操作の組み合わせ
   - エラーハンドリング

5. **benches/README.md** - ベンチマークの詳細ドキュメント

6. **Cargo.toml** - 更新内容
   - `criterion` クレートを dev-dependencies に追加
   - 3つのベンチマーク設定を追加

## セットアップ手順

### Windows環境（推奨）

このプロジェクトはWindows向けアプリケーションのため、Windows環境で実行することを推奨します。

```powershell
# ベンチマークの実行
cargo bench

# 特定のベンチマークのみ実行
cargo bench --bench search_benchmark
cargo bench --bench alias_benchmark
cargo bench --bench file_operations_benchmark
```

### WSL/Linux環境

WSL環境で実行する場合、以下のシステムライブラリが必要です：

```bash
# 必要なライブラリをインストール
sudo apt-get update
sudo apt-get install -y libxdo-dev libgtk-3-dev

# ベンチマークの実行
cargo bench
```

## ベンチマーク結果の確認

### コンソール出力

ベンチマーク実行時、以下のような統計情報が表示されます：

```
search_exact_match      time:   [45.123 µs 46.456 µs 47.789 µs]
                        change: [-2.3456% -1.2345% +0.1234%] (p = 0.05 < 0.05)
                        Performance has improved.
```

### HTML レポート

詳細なグラフとレポートは以下の場所に生成されます：

```
target/criterion/report/index.html
```

ブラウザで開くと、以下の情報が確認できます：

- 実行時間の推移グラフ
- パーセンタイル分布
- 前回実行との比較
- 統計的有意性の検定結果

## 検証方法

### 1. ベンチマークがコンパイルできることを確認

```bash
cargo bench --no-run
```

成功メッセージ例：
```
Finished `bench` profile [optimized] target(s) in X.XXs
```

### 2. 個別ベンチマークの実行

```bash
# 検索処理のベンチマーク
cargo bench --bench search_benchmark

# エイリアス管理のベンチマーク
cargo bench --bench alias_benchmark

# ファイル操作のベンチマーク
cargo bench --bench file_operations_benchmark
```

### 3. 特定のテストのみ実行

```bash
# 完全一致検索のみ
cargo bench --bench search_benchmark -- exact_match

# スケーラビリティテストのみ
cargo bench --bench search_benchmark -- scalability
```

## パフォーマンス目標

### 検索処理
- 100件のデータで完全一致検索: < 100 µs
- 100件のデータでファジーマッチ: < 500 µs
- 1000件のデータで検索: < 5 ms
- キャッシュヒット: < 10 µs

### エイリアス管理
- 単一エイリアス追加: < 10 µs
- 100件エイリアス追加: < 5 ms
- エイリアス削除: < 50 µs
- エイリアス更新: < 50 µs

### ファイル操作
- 1KB ファイルコピー: < 1 ms
- 100KB ファイルコピー: < 10 ms
- 1MB ファイルコピー: < 100 ms
- ファイル移動: < 5 ms
- ファイル削除: < 5 ms

## ベンチマークの特徴

### 実際のデータを使用

各ベンチマークは実際の使用パターンを模倣したテストデータを使用：

- **検索**: 日本語エイリアス、階層パス、タグ情報を含む
- **エイリアス**: UUID生成、タイムスタンプ、重複チェック
- **ファイル操作**: tempfile を使用した実ファイル操作

### 複数のテストケース

- **小規模データ**: 基本的な操作の性能測定
- **中規模データ**: 実用的なデータ量での性能測定
- **大規模データ**: スケーラビリティの確認

### 統計的な精度

`criterion` クレートの機能により：

- 複数回実行して平均値を計算
- 外れ値の検出と除外
- 統計的有意性の検定
- 前回実行との比較

## トラブルシューティング

### エラー: "unable to find library -lxdo"

**原因**: Linux環境で `global-hotkey` クレートが必要とするシステムライブラリが不足

**解決方法**:
```bash
sudo apt-get install libxdo-dev libgtk-3-dev
```

または、Windows環境で実行してください。

### エラー: "could not compile `ofkt`"

**原因**: ビルド時の依存関係の問題

**解決方法**:
```bash
# キャッシュをクリア
cargo clean

# 再ビルド
cargo build --lib --release

# ベンチマーク実行
cargo bench
```

### ベンチマーク実行が遅い

**原因**: デバッグビルドで実行されている可能性

**解決方法**:
```bash
# release プロファイルで実行（自動的に使用されますが、明示的に指定も可能）
cargo bench --release
```

## 次のステップ

### ボトルネックの特定

ベンチマーク結果から性能のボトルネックを特定：

1. 最も時間がかかっている処理を確認
2. データ量による性能変化を分析
3. キャッシュの効果を評価

### 最適化の実施

特定されたボトルネックに対して最適化：

1. アルゴリズムの改善
2. データ構造の見直し
3. キャッシュ戦略の調整
4. 並列処理の導入

### 継続的な測定

最適化後、ベンチマークを再実行して効果を測定：

```bash
# ベースライン確定（最適化前）
cargo bench -- --save-baseline before

# 最適化実施後
cargo bench -- --baseline before
```

これにより、最適化の効果が数値で確認できます。
