# Ofkt - Windows常駐型ファイル管理ツール

Windowsエクスプローラーの代替となる軽量な常駐型ファイル管理ツールです。

## プロジェクト概要

Ofkt（Overflow Filer Kaiten）は、頻繁に使用するフォルダやファイルに素早くアクセスするための常駐型ツールです。エイリアス機能により、深い階層のフォルダにも瞬時にアクセスでき、日々の作業効率を大幅に向上させます。

## 主要機能

### コア機能
- ✅ **グローバルホットキー（カスタマイズ可能）** - デフォルト: Ctrl+Shift+Oでウィンドウ表示/非表示
- ✅ **画面端トリガー** - 画面右端（1ピクセル）にカーソル移動で自動表示
- ✅ **システムトレイ常駐** - タスクトレイに常駐し、必要な時だけ呼び出せる
- ✅ **Windows起動時の自動起動** - ログイン時に自動で起動

### 検索・アクセス機能
- ✅ **フォルダエイリアス設定** - 別名で検索・アクセス（例: "会計" → "C:\Documents\2025年度\経理\会計資料"）
- ✅ **階層構造検索** - "試算表 202506" → "2025年度/会計/試算表/202506"のように階層を認識
- ✅ **インクリメンタル検索** - 入力しながらリアルタイムで絞り込み
- ✅ **ファジーマッチング検索** - 曖昧な入力でも適切な候補を提案
- ✅ **タグ機能** - エイリアスにタグを付けて分類・検索
- ✅ **カラーラベル** - フォルダに色を付けて視覚的に識別

### 履歴・お気に入り
- ✅ **履歴機能** - 最近開いたファイル・フォルダを自動記録
- ✅ **お気に入り機能** - 頻繁に使うフォルダをスター付けで上部固定
- ✅ **スコアリングシステム** - お気に入りと最終アクセス日時でランキング

### ファイル操作
- ✅ **ツリー形式のフォルダ表示** - 階層構造を直感的に表示
- ✅ **ファイル操作** - 開く、コピー、移動、削除、名前変更
- ✅ **ゴミ箱対応** - 削除時にゴミ箱に移動（完全削除も可能）
- ✅ **エラーハンドリング** - 日本語のわかりやすいエラーメッセージ

### UI・カスタマイズ
- ✅ **システムテーマ自動検出** - Windows設定に応じてダーク/ライトモード自動切替
- ✅ **手動テーマ選択** - ライト/ダーク/システム連動から選択可能
- ✅ **アプリ内GUI設定画面** - すべての設定をGUIから変更可能
- ✅ **ウィンドウサイズ調整** - ドラッグで自由にリサイズ可能

## インストール

### 必要要件

- OS: Windows 10/11（64bit）
- メモリ: 最小 100MB
- ディスク: 約 10MB

### ビルド済みバイナリ（現在開発中）

リリース版は現在準備中です。将来的にGitHubのReleasesページからダウンロード可能になります。

### ソースからビルド

#### 前提条件
- Rust 1.70以降（最新の安定版を推奨）
- Windows環境（WSLではビルドできません）

#### ビルド手順

1. **リポジトリのクローン**
   ```bash
   git clone https://github.com/yourusername/ofkt.git
   cd ofkt
   ```

2. **依存関係の確認**
   ```bash
   cargo check
   ```

3. **デバッグビルド（開発用）**
   ```bash
   cargo build
   ```
   実行ファイル: `target/debug/ofkt.exe`

4. **リリースビルド（本番用）**
   ```bash
   cargo build --release
   ```
   実行ファイル: `target/release/ofkt.exe`

## 使い方

### 基本操作

#### 1. 起動
- `ofkt.exe`をダブルクリックして起動
- システムトレイにアイコンが表示されます

#### 2. ウィンドウの表示/非表示
以下の3つの方法でウィンドウを表示できます：
- **ホットキー**: `Ctrl+Shift+O`（カスタマイズ可能）
- **画面端トリガー**: マウスカーソルを画面右端に300ms以上置く
- **トレイアイコン**: システムトレイのアイコンを左クリック

#### 3. エイリアスの追加
1. 設定画面を開く（システムトレイ → 設定、または設定タブ）
2. 「エイリアス管理」セクションで「新規追加」をクリック
3. 以下を入力：
   - **名前**: エイリアスの名前（例: "会計"）
   - **パス**: 実際のフォルダパス（例: "C:\Documents\2025年度\経理\会計資料"）
   - **タグ**: カンマ区切りで分類タグ（例: "仕事, 経理"）
   - **色**: 視覚的な識別のための色（オプション）

#### 4. 検索
1. メインウィンドウの検索バーに入力
2. リアルタイムで結果が絞り込まれます
3. 以下の検索方法に対応：
   - **完全一致**: エイリアス名と完全に一致
   - **前方一致**: エイリアス名の前方部分が一致
   - **ファジーマッチ**: 曖昧な入力でもスコアリングして提案
   - **タグ検索**: タグを含む検索
   - **階層検索**: "プロジェクト 2025 設計書"のように階層を含む検索

#### 5. ファイル操作
- **開く**: エイリアスをダブルクリック、または右クリック → 開く
- **コピー**: 右クリック → コピー
- **移動**: 右クリック → 移動
- **削除**: 右クリック → 削除（ゴミ箱へ）
- **名前変更**: 右クリック → 名前変更

#### 6. お気に入り
- エイリアスの横にある星アイコンをクリックでお気に入り登録
- お気に入りは検索結果の上位に表示されます

### 設定

設定画面（システムトレイ → 設定、または設定タブ）で以下をカスタマイズできます：

- **ホットキー**: キーの組み合わせを変更
- **画面端トリガー**: 有効/無効、検出時間（デフォルト300ms）
- **自動起動**: Windows起動時の自動起動設定
- **テーマ**: ライト/ダーク/システム連動
- **ウィンドウサイズ**: 幅・高さの初期値
- **検索設定**: 最大結果数、キャッシュサイズ
- **ファイル操作**: デフォルトで削除時にゴミ箱を使用するか

## ビルド方法

### デバッグビルド
```bash
cargo build
```

### リリースビルド
```bash
cargo build --release
```

リリースビルドでは以下の最適化が適用されます：
- サイズ最適化（`opt-level = "z"`）
- LTO（Link Time Optimization）
- シンボル情報の削除（`strip = true`）

### テスト実行
```bash
# 全テスト実行
cargo test

# ライブラリテストのみ
cargo test --lib

# 特定モジュールのテスト
cargo test --lib core::search
```

### ベンチマーク実行（Windows環境のみ）
```bash
cargo bench
```

## 設定ファイル

設定ファイルは以下の場所に保存されます：

- **Windows**: `%APPDATA%\ofkt\`
  - 設定: `config.json`
  - エイリアス: `aliases.json`

設定ファイルは自動的に作成されます。手動で編集することも可能ですが、アプリ内のGUI設定画面からの編集を推奨します。

## トラブルシューティング

### ホットキーが動作しない
- 他のアプリケーションと競合している可能性があります
- 設定画面で別のキーの組み合わせに変更してください

### 画面端トリガーが反応しない
- Windows環境でのみ動作します（WSLでは非対応）
- 設定画面で有効になっているか確認してください
- 検出時間（デフォルト300ms）を調整してみてください

### システムトレイアイコンが表示されない
- Windowsのシステムトレイ設定を確認してください
- タスクマネージャーでプロセスが起動しているか確認してください

### アプリケーションが起動しない
- WSL環境では実行できません（Windows専用）
- エラーログを確認してください（実装予定）

詳細なトラブルシューティングは[USER_GUIDE.md](USER_GUIDE.md)を参照してください。

## パフォーマンス

- **検索速度**: 1,000件のエイリアスでも50µs以下
- **起動時間**: 100ms以下（遅延初期化により体感速度を向上）
- **メモリ使用量**: 約30-50MB（エイリアス1,000件時）
- **キャッシュ**: LRUキャッシュにより頻繁な検索を高速化

詳細は[OPTIMIZATION_REPORT.md](OPTIMIZATION_REPORT.md)を参照してください。

## テスト

### テストカバレッジ
- **総テスト数**: 103テスト
- **成功率**: 100%
- **カバレッジ**: コアモジュール 85-95%+

詳細は[TEST_COVERAGE.md](TEST_COVERAGE.md)を参照してください。

## ライセンス

MIT License

Copyright (c) 2026 Ofkt Project

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## 貢献

現在、このプロジェクトは開発中です。バグ報告や機能提案は、GitHubのIssuesで受け付けています。

## 開発者向けドキュメント

- [TEST_COVERAGE.md](TEST_COVERAGE.md) - テストカバレッジレポート
- [OPTIMIZATION_REPORT.md](OPTIMIZATION_REPORT.md) - パフォーマンス最適化レポート
- [INTEGRATION_TEST.md](INTEGRATION_TEST.md) - 統合テスト手順書
- [BUGFIX_LOG.md](BUGFIX_LOG.md) - バグフィックスログ
- [PHASE_6_COMPLETE.md](PHASE_6_COMPLETE.md) - Phase 6 完了レポート

## バージョン履歴

[CHANGELOG.md](CHANGELOG.md)を参照してください。

## サポート

- GitHub Issues: バグ報告・機能提案
- Email: (準備中)

---

**作成日**: 2026-01-25
**バージョン**: 0.1.0
