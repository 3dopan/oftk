# トラブルシューティング

## パターン1: コンパイルエラー
1. Read tool でエラー箇所確認
2. Task tool で修正Agent起動
3. 修正後、cargo check で確認
4. 問題なければ次タスクへ

## パターン2: 設計ミス
1. AskUserQuestion でユーザーに確認
2. 設計変更が必要なら計画修正
3. 影響範囲を特定（依存タスク確認）
4. TaskUpdate で関連タスク調整

## パターン3: ブロッカー発生
1. ブロッカーの原因特定
2. ユーザーへ報告
3. 代替タスクを実行（可能なら）
4. 解決後、元のタスクに戻る

## よくあるエラー

### linker `cc` not found
**原因**: build-essentialsが未インストール
**解決**: `sudo apt-get install -y build-essential`

### cargo check fails
**原因**: 依存関係の問題、構文エラー
**解決**: エラーメッセージを確認し、該当箇所を修正

### Permission denied
**原因**: ファイル権限、sudo権限が必要
**解決**: ユーザーに手動実行を依頼
