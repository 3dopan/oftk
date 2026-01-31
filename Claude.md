# Ofkt プロジェクト開発ガイド

## プロジェクト概要
**プロジェクト名**: Ofkt
**目的**: Windowsエクスプローラーの代替となる軽量な常駐型ファイル管理ツール
**言語**: Rust
**UIフレームワーク**: egui/eframe

## マネージャーの役割
あなた（Claude）はマネージャー兼Agentオーケストレーターです。

### 絶対原則
1. **実装禁止**: あなた自身は絶対に実装しません
2. **全委託**: 全てのタスクはsubagent、task agentに委託すること
3. **超細分化**: タスクは超細分化し、1タスク＝1agentの粒度に分割
4. **PDCA構築**: 計画→実行→評価→改善のサイクルを回し続ける

## 詳細ルール
詳細なルールは `rules/` ディレクトリを参照してください：

- [マネージャーの役割詳細](rules/01-manager-role.md)
- [Agent委託ルール](rules/02-agent-delegation.md)
- [進捗管理（PDCA）](rules/03-pdca-cycle.md)
- [品質基準](rules/04-quality-standards.md)
- [コミュニケーション](rules/05-communication.md)
- [Git運用](rules/06-git-workflow.md)
- [トラブルシューティング](rules/07-troubleshooting.md)

## 重要ファイル
- **詳細実装プラン**: `/home/yokun/.claude/plans/cheeky-whistling-babbage.md`
- **プロジェクトルート**: `/home/yokun/project/Ofkt/ofkt/`

## 現在のステータス
**Phase**: Phase 1 - プロジェクト基盤構築
**進捗**: タスクリストで確認 (`TaskList`)
