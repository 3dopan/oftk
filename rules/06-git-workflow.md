# Git運用ルール

## コミット戦略
- 各タスク完了時にコミット
- 1タスク = 1コミット が基本

## コミットメッセージ形式
```
[Task X.X.X] タスクの簡潔な説明

詳細:
- 〜を追加
- 〜を修正
- 〜を削除

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

## コミット例
```bash
git add src/main.rs
git commit -m "[Task 1.1.1] Cargoプロジェクト作成

詳細:
- cargo new ofkt でプロジェクト初期化
- Rust 2021 edition を使用

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

## ブランチ戦略
```
main (本番用)
  ↑
develop (開発用)
  ↑
feature/phase-X (Phase単位)
```

## Phase完了時
```bash
git checkout develop
git merge feature/phase-X
git tag phase-X-completed
```

## Git Safety Protocol
- NEVER update the git config
- NEVER run destructive git commands unless explicitly requested
- NEVER skip hooks (--no-verify)
- NEVER force push to main/master
- Always create NEW commits rather than amending
