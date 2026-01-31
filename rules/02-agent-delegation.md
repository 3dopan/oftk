# Agent委託ルール

## タスク実行フロー
```
1. TaskCreate で次のタスクを作成
2. TaskUpdate でステータスを in_progress に変更
3. Task tool でagentを起動（subagent_type選択）
4. Agent完了を待つ
5. Read tool で成果物をレビュー
6. Bash tool で動作確認（cargo check等）
7. 問題なければ TaskUpdate で completed に変更
8. 問題あれば再度Agentに修正依頼
```

## Subagent選択基準
| タスク種類 | Subagent Type | 例 |
|-----------|--------------|-----|
| ファイル作成・編集 | general-purpose | Cargo.toml作成、コード実装 |
| コード実装 | general-purpose | 構造体定義、関数実装 |
| バグ調査 | Explore | エラー原因調査、依存関係確認 |
| テスト実行 | Bash | cargo test, cargo run |

## タスク委託プロンプト形式
```markdown
【タスクID】Task X.X.X
【Phase】Phase X
【目的】〜を実装する

【詳細】
- 〜を作成
- 〜を実装

【成果物】
- ファイルパス: 〜

【検証方法】
1. cargo check でコンパイル確認
2. （あれば）cargo test でテスト確認

【参照】
- 設計書: プランファイル参照

【制約】
- （あれば）制約事項
```
