//! 操作履歴管理モジュール
//!
//! ファイル操作（削除、移動、コピー、リネーム）の履歴を管理し、
//! Undo/Redo機能を提供します。

use std::path::PathBuf;

/// ファイル操作の種類
#[derive(Debug, Clone)]
pub enum FileOperation {
    /// ファイル/フォルダの削除（ゴミ箱への移動）
    Delete {
        /// 削除されたファイルの元のパス
        original_path: PathBuf,
    },
    /// ファイル/フォルダの移動
    Move {
        /// 移動元のパス
        source: PathBuf,
        /// 移動先のパス
        destination: PathBuf,
    },
    /// ファイル/フォルダのコピー
    Copy {
        /// コピー元のパス
        source: PathBuf,
        /// コピー先のパス
        destination: PathBuf,
    },
    /// ファイル/フォルダの名前変更
    Rename {
        /// 変更前のパス
        old_path: PathBuf,
        /// 変更後のパス
        new_path: PathBuf,
    },
}

impl FileOperation {
    /// 操作の説明を取得
    pub fn description(&self) -> String {
        match self {
            FileOperation::Delete { original_path, .. } => {
                format!("削除: {}", original_path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| original_path.display().to_string()))
            }
            FileOperation::Move { source, destination } => {
                format!("移動: {} -> {}",
                    source.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
                    destination.display())
            }
            FileOperation::Copy { destination, .. } => {
                format!("コピー: {}", destination.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default())
            }
            FileOperation::Rename { old_path, new_path } => {
                format!("名前変更: {} -> {}",
                    old_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
                    new_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default())
            }
        }
    }
}

/// 操作履歴マネージャー
#[derive(Debug)]
pub struct OperationHistoryManager {
    /// 操作履歴スタック（LIFO）
    history: Vec<FileOperation>,
    /// Redo用スタック
    redo_stack: Vec<FileOperation>,
    /// 最大履歴数
    max_entries: usize,
}

impl Default for OperationHistoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl OperationHistoryManager {
    /// 新しい OperationHistoryManager を作成
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            redo_stack: Vec::new(),
            max_entries: 50,
        }
    }

    /// 操作を履歴に追加
    pub fn push(&mut self, operation: FileOperation) {
        self.history.push(operation);
        // 最大履歴数を超えた場合、古いエントリを削除
        while self.history.len() > self.max_entries {
            self.history.remove(0);
        }
        // 新しい操作を追加したらRedoスタックをクリア
        self.redo_stack.clear();
    }

    /// Undo: 最後の操作を取り消す
    pub fn undo(&mut self) -> Result<String, String> {
        let operation = self.history.pop()
            .ok_or_else(|| "取り消す操作がありません".to_string())?;

        let result = self.execute_undo(&operation)?;
        self.redo_stack.push(operation);
        Ok(result)
    }

    /// Redo: 取り消した操作をやり直す
    pub fn redo(&mut self) -> Result<String, String> {
        let operation = self.redo_stack.pop()
            .ok_or_else(|| "やり直す操作がありません".to_string())?;

        let result = self.execute_redo(&operation)?;
        self.history.push(operation);
        Ok(result)
    }

    /// Undoが可能かどうか
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    /// Redoが可能かどうか
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Undo操作の実行
    fn execute_undo(&self, operation: &FileOperation) -> Result<String, String> {
        match operation {
            FileOperation::Delete { original_path, .. } => {
                // ゴミ箱からの復元は難しいので、メッセージのみ
                Err(format!("「{}」の削除は取り消せません（ゴミ箱から手動で復元してください）",
                    original_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()))
            }
            FileOperation::Move { source, destination } => {
                // 移動の逆: destination から source に戻す
                if destination.exists() {
                    std::fs::rename(destination, source)
                        .map_err(|e| format!("移動の取り消しに失敗: {}", e))?;
                    Ok(format!("移動を取り消しました: {} に戻しました",
                        source.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()))
                } else {
                    Err("移動先のファイルが見つかりません".to_string())
                }
            }
            FileOperation::Copy { destination, .. } => {
                // コピーの逆: destination を削除
                if destination.exists() {
                    if destination.is_dir() {
                        std::fs::remove_dir_all(destination)
                            .map_err(|e| format!("コピーの取り消しに失敗: {}", e))?;
                    } else {
                        std::fs::remove_file(destination)
                            .map_err(|e| format!("コピーの取り消しに失敗: {}", e))?;
                    }
                    Ok(format!("コピーを取り消しました: {} を削除しました",
                        destination.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()))
                } else {
                    Err("コピー先のファイルが見つかりません".to_string())
                }
            }
            FileOperation::Rename { old_path, new_path } => {
                // リネームの逆: new_path から old_path に戻す
                if new_path.exists() {
                    std::fs::rename(new_path, old_path)
                        .map_err(|e| format!("名前変更の取り消しに失敗: {}", e))?;
                    Ok(format!("名前を元に戻しました: {}",
                        old_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()))
                } else {
                    Err("変更後のファイルが見つかりません".to_string())
                }
            }
        }
    }

    /// Redo操作の実行
    fn execute_redo(&self, operation: &FileOperation) -> Result<String, String> {
        match operation {
            FileOperation::Delete { .. } => {
                Err("削除のやり直しはサポートされていません".to_string())
            }
            FileOperation::Move { source, destination } => {
                if source.exists() {
                    std::fs::rename(source, destination)
                        .map_err(|e| format!("移動のやり直しに失敗: {}", e))?;
                    Ok(format!("移動をやり直しました"))
                } else {
                    Err("移動元のファイルが見つかりません".to_string())
                }
            }
            FileOperation::Copy { source, destination } => {
                if source.exists() {
                    if source.is_dir() {
                        // ディレクトリのコピーは複雑なので簡略化
                        Err("ディレクトリのコピーやり直しはサポートされていません".to_string())
                    } else {
                        std::fs::copy(source, destination)
                            .map_err(|e| format!("コピーのやり直しに失敗: {}", e))?;
                        Ok(format!("コピーをやり直しました"))
                    }
                } else {
                    Err("コピー元のファイルが見つかりません".to_string())
                }
            }
            FileOperation::Rename { old_path, new_path } => {
                if old_path.exists() {
                    std::fs::rename(old_path, new_path)
                        .map_err(|e| format!("名前変更のやり直しに失敗: {}", e))?;
                    Ok(format!("名前変更をやり直しました"))
                } else {
                    Err("ファイルが見つかりません".to_string())
                }
            }
        }
    }

    /// 履歴をクリア
    pub fn clear(&mut self) {
        self.history.clear();
        self.redo_stack.clear();
    }
}
