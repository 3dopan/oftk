use std::fs;
use std::path::Path;

#[cfg(target_os = "windows")]
use std::process::Command;

/// ファイル操作管理
///
/// ファイルの基本的な操作機能を提供します。
/// - ファイルを開く
/// - ファイルをコピー
/// - ファイルを移動
/// - ファイルを削除
/// - ファイル名を変更
pub struct FileManager {
    // 将来の拡張のためのフィールド
    // 例: 操作履歴、設定など
}

impl FileManager {
    /// 新しい FileManager を作成
    pub fn new() -> Self {
        Self {}
    }

    /// ファイル/フォルダをデフォルトアプリケーションで開く
    ///
    /// # 引数
    /// * `path` - 開くファイルまたはフォルダのパス
    ///
    /// # 戻り値
    /// * `Ok(())` - 成功
    /// * `Err(String)` - エラーメッセージ
    ///
    /// # 例
    /// ```no_run
    /// use ofkt::core::FileManager;
    /// use std::path::Path;
    ///
    /// let manager = FileManager::new();
    /// manager.open(Path::new("C:\\Users\\test.txt")).unwrap();
    /// ```
    pub fn open(&self, path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("パス '{}' は存在しません", path.display()));
        }

        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(&["/C", "start", "", path.to_str().unwrap()])
                .spawn()
                .map_err(|e| format!("ファイルを開けません: {}", e))?;
            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err("Windows環境でのみサポートされています".to_string())
        }
    }

    /// ファイルをコピー
    ///
    /// # 引数
    /// * `src` - コピー元のパス
    /// * `dest` - コピー先のパス
    ///
    /// # 戻り値
    /// * `Ok(())` - 成功
    /// * `Err(String)` - エラーメッセージ
    ///
    /// # 例
    /// ```no_run
    /// use ofkt::core::FileManager;
    /// use std::path::Path;
    ///
    /// let manager = FileManager::new();
    /// manager.copy(
    ///     Path::new("C:\\Users\\source.txt"),
    ///     Path::new("C:\\Users\\dest.txt")
    /// ).unwrap();
    /// ```
    pub fn copy(&self, src: &Path, dest: &Path) -> Result<(), String> {
        if !src.exists() {
            return Err(format!("コピー元 '{}' は存在しません", src.display()));
        }

        // 宛先の親ディレクトリが存在するか確認
        if let Some(parent) = dest.parent() {
            if !parent.exists() {
                return Err(format!(
                    "宛先ディレクトリ '{}' は存在しません",
                    parent.display()
                ));
            }
        }

        fs::copy(src, dest)
            .map_err(|e| format!("コピー失敗: {}", e))?;

        Ok(())
    }

    /// ファイルを移動
    ///
    /// # 引数
    /// * `src` - 移動元のパス
    /// * `dest` - 移動先のパス
    ///
    /// # 戻り値
    /// * `Ok(())` - 成功
    /// * `Err(String)` - エラーメッセージ
    ///
    /// # 例
    /// ```no_run
    /// use ofkt::core::FileManager;
    /// use std::path::Path;
    ///
    /// let manager = FileManager::new();
    /// manager.move_file(
    ///     Path::new("C:\\Users\\source.txt"),
    ///     Path::new("C:\\Users\\dest.txt")
    /// ).unwrap();
    /// ```
    pub fn move_file(&self, src: &Path, dest: &Path) -> Result<(), String> {
        log::debug!("move_file開始: {} -> {}", src.display(), dest.display());

        if !src.exists() {
            log::error!("移動失敗: 移動元が存在しません - {}", src.display());
            return Err(format!("移動元 '{}' は存在しません", src.display()));
        }

        // 宛先の親ディレクトリが存在するか確認
        if let Some(parent) = dest.parent() {
            if !parent.exists() {
                log::error!("移動失敗: 宛先ディレクトリが存在しません - {}", parent.display());
                return Err(format!(
                    "宛先ディレクトリ '{}' は存在しません",
                    parent.display()
                ));
            }
        }

        // fs::rename を試みる（クロスデバイス移動が失敗する可能性あり）
        log::debug!("fs::renameを試行中...");
        match fs::rename(src, dest) {
            Ok(_) => {
                log::info!("move_file完了（fs::rename成功）: {} -> {}", src.display(), dest.display());
                Ok(())
            },
            Err(e) => {
                // クロスデバイス移動の場合はコピー&削除で対応
                if e.raw_os_error() == Some(17) || e.kind() == std::io::ErrorKind::CrossesDevices {
                    log::warn!("クロスデバイス移動を検出、コピー&削除モードに切り替え: {:?}", e.kind());

                    log::debug!("ステップ1: ファイルコピー中...");
                    fs::copy(src, dest)
                        .map_err(|e| {
                            log::error!("移動失敗（コピーフェーズ）: {}", e);
                            format!("移動失敗（コピー）: {}", e)
                        })?;

                    log::debug!("ステップ2: 元ファイル削除中...");
                    if src.is_dir() {
                        fs::remove_dir_all(src)
                            .map_err(|e| {
                                log::error!("移動失敗（削除フェーズ - ディレクトリ）: {}", e);
                                format!("移動失敗（削除）: {}", e)
                            })?;
                    } else {
                        fs::remove_file(src)
                            .map_err(|e| {
                                log::error!("移動失敗（削除フェーズ - ファイル）: {}", e);
                                format!("移動失敗（削除）: {}", e)
                            })?;
                    }
                    log::info!("move_file完了（クロスデバイス移動）: {} -> {}", src.display(), dest.display());
                    Ok(())
                } else {
                    log::error!("move_file失敗: {} - エラー: {:?} (raw_os_error: {:?})",
                        src.display(), e.kind(), e.raw_os_error());
                    Err(format!("移動失敗: {}", e))
                }
            }
        }
    }

    /// ファイルを削除（ゴミ箱へ移動 or 完全削除）
    ///
    /// # 引数
    /// * `path` - 削除するファイルまたはフォルダのパス
    /// * `permanent` - true の場合は完全削除、false の場合はゴミ箱へ移動
    ///
    /// # 戻り値
    /// * `Ok(())` - 成功
    /// * `Err(String)` - エラーメッセージ
    ///
    /// # 例
    /// ```no_run
    /// use ofkt::core::FileManager;
    /// use std::path::Path;
    ///
    /// let manager = FileManager::new();
    /// // ゴミ箱へ移動
    /// manager.delete(Path::new("C:\\Users\\test.txt"), false).unwrap();
    /// // 完全削除
    /// manager.delete(Path::new("C:\\Users\\test2.txt"), true).unwrap();
    /// ```
    pub fn delete(&self, path: &Path, permanent: bool) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("削除対象 '{}' は存在しません", path.display()));
        }

        if permanent {
            if path.is_dir() {
                fs::remove_dir_all(path)
                    .map_err(|e| format!("削除失敗: {}", e))?;
            } else {
                fs::remove_file(path)
                    .map_err(|e| format!("削除失敗: {}", e))?;
            }
        } else {
            trash::delete(path)
                .map_err(|e| format!("ゴミ箱への移動失敗: {}", e))?;
        }

        Ok(())
    }

    /// ファイル名を変更
    ///
    /// # 引数
    /// * `path` - 名前を変更するファイルまたはフォルダのパス
    /// * `new_name` - 新しい名前（パスではなくファイル名のみ）
    ///
    /// # 戻り値
    /// * `Ok(())` - 成功
    /// * `Err(String)` - エラーメッセージ
    ///
    /// # 例
    /// ```no_run
    /// use ofkt::core::FileManager;
    /// use std::path::Path;
    ///
    /// let manager = FileManager::new();
    /// manager.rename(Path::new("C:\\Users\\old.txt"), "new.txt").unwrap();
    /// ```
    pub fn rename(&self, path: &Path, new_name: &str) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("対象 '{}' は存在しません", path.display()));
        }

        let parent = path
            .parent()
            .ok_or_else(|| "親ディレクトリが見つかりません".to_string())?;
        let new_path = parent.join(new_name);

        fs::rename(path, new_path)
            .map_err(|e| format!("名前変更失敗: {}", e))?;

        Ok(())
    }

    /// ファイルまたはディレクトリを再帰的にコピー
    ///
    /// # 引数
    /// * `src` - コピー元のパス
    /// * `dest` - コピー先のパス
    ///
    /// # 戻り値
    /// * `Ok(())` - 成功
    /// * `Err(String)` - エラーメッセージ
    ///
    /// # 例
    /// ```no_run
    /// use ofkt::core::FileManager;
    /// use std::path::Path;
    ///
    /// let manager = FileManager::new();
    /// manager.copy_recursive(
    ///     Path::new("C:\\Users\\source_dir"),
    ///     Path::new("C:\\Users\\dest_dir")
    /// ).unwrap();
    /// ```
    pub fn copy_recursive(&self, src: &Path, dest: &Path) -> Result<(), String> {
        log::debug!("copy_recursive開始: {} -> {}", src.display(), dest.display());
        self.copy_recursive_internal(src, dest, 0)
    }

    fn copy_recursive_internal(&self, src: &Path, dest: &Path, depth: usize) -> Result<(), String> {
        if src.is_dir() {
            // ディレクトリの場合
            log::debug!("[深度:{}] ディレクトリコピー: {} -> {}", depth, src.display(), dest.display());
            std::fs::create_dir_all(dest)
                .map_err(|e| {
                    log::error!("[深度:{}] ディレクトリ作成失敗: {} - エラー: {}", depth, dest.display(), e);
                    format!("ディレクトリ作成失敗: {}", e)
                })?;

            for entry in std::fs::read_dir(src)
                .map_err(|e| {
                    log::error!("[深度:{}] ディレクトリ読み込み失敗: {} - エラー: {}", depth, src.display(), e);
                    format!("ディレクトリ読み込み失敗: {}", e)
                })?
            {
                let entry = entry.map_err(|e| {
                    log::error!("[深度:{}] エントリ読み込み失敗: エラー: {}", depth, e);
                    format!("エントリ読み込み失敗: {}", e)
                })?;
                let src_path = entry.path();
                let dest_path = dest.join(entry.file_name());

                self.copy_recursive_internal(&src_path, &dest_path, depth + 1)?;
            }

            log::debug!("[深度:{}] ディレクトリコピー完了: {}", depth, src.display());
            Ok(())
        } else {
            // ファイルの場合
            log::debug!("[深度:{}] ファイルコピー: {} -> {}", depth, src.display(), dest.display());
            let result = self.copy(src, dest);
            if let Err(ref e) = result {
                log::error!("[深度:{}] ファイルコピー失敗: {} - エラー: {}", depth, src.display(), e);
            } else {
                log::debug!("[深度:{}] ファイルコピー完了: {}", depth, src.display());
            }
            result
        }
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_new() {
        let _manager = FileManager::new();
        // 構造体が正常に作成されることを確認
        // 現時点ではフィールドがないため、インスタンス化できるかのみ確認
    }

    #[test]
    fn test_default() {
        let _manager = FileManager::default();
        // Default トレイトが正常に動作することを確認
    }

    #[test]
    fn test_copy() {
        let manager = FileManager::new();
        let temp_dir = tempdir().unwrap();

        // テストファイルを作成
        let src_path = temp_dir.path().join("source.txt");
        let mut file = File::create(&src_path).unwrap();
        writeln!(file, "テストデータ").unwrap();

        // コピー先パス
        let dest_path = temp_dir.path().join("dest.txt");

        // コピーを実行
        let result = manager.copy(&src_path, &dest_path);
        assert!(result.is_ok());

        // 両方のファイルが存在することを確認
        assert!(src_path.exists());
        assert!(dest_path.exists());

        // 内容が同じことを確認
        let src_content = fs::read_to_string(&src_path).unwrap();
        let dest_content = fs::read_to_string(&dest_path).unwrap();
        assert_eq!(src_content, dest_content);
    }

    #[test]
    fn test_copy_nonexistent_source() {
        let manager = FileManager::new();
        let temp_dir = tempdir().unwrap();

        let src_path = temp_dir.path().join("nonexistent.txt");
        let dest_path = temp_dir.path().join("dest.txt");

        // 存在しないファイルのコピーはエラーになる
        let result = manager.copy(&src_path, &dest_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("存在しません"));
    }

    #[test]
    fn test_copy_to_nonexistent_directory() {
        let manager = FileManager::new();
        let temp_dir = tempdir().unwrap();

        // テストファイルを作成
        let src_path = temp_dir.path().join("source.txt");
        File::create(&src_path).unwrap();

        // 存在しないディレクトリへのコピー
        let dest_path = temp_dir.path().join("nonexistent_dir").join("dest.txt");

        let result = manager.copy(&src_path, &dest_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("宛先ディレクトリ"));
    }

    #[test]
    fn test_move_file() {
        let manager = FileManager::new();
        let temp_dir = tempdir().unwrap();

        // テストファイルを作成
        let src_path = temp_dir.path().join("source.txt");
        let mut file = File::create(&src_path).unwrap();
        writeln!(file, "移動するデータ").unwrap();

        // 移動先パス
        let dest_path = temp_dir.path().join("dest.txt");

        // 移動を実行
        let result = manager.move_file(&src_path, &dest_path);
        assert!(result.is_ok());

        // 移動元が存在しないことを確認
        assert!(!src_path.exists());

        // 移動先が存在することを確認
        assert!(dest_path.exists());

        // 内容が保持されていることを確認
        let content = fs::read_to_string(&dest_path).unwrap();
        assert!(content.contains("移動するデータ"));
    }

    #[test]
    fn test_move_file_nonexistent_source() {
        let manager = FileManager::new();
        let temp_dir = tempdir().unwrap();

        let src_path = temp_dir.path().join("nonexistent.txt");
        let dest_path = temp_dir.path().join("dest.txt");

        // 存在しないファイルの移動はエラーになる
        let result = manager.move_file(&src_path, &dest_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("存在しません"));
    }

    #[test]
    fn test_move_file_to_nonexistent_directory() {
        let manager = FileManager::new();
        let temp_dir = tempdir().unwrap();

        // テストファイルを作成
        let src_path = temp_dir.path().join("source.txt");
        File::create(&src_path).unwrap();

        // 存在しないディレクトリへの移動
        let dest_path = temp_dir.path().join("nonexistent_dir").join("dest.txt");

        let result = manager.move_file(&src_path, &dest_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("宛先ディレクトリ"));
    }

    #[test]
    fn test_delete_permanent_file() {
        let manager = FileManager::new();
        let temp_dir = tempdir().unwrap();

        // テストファイルを作成
        let file_path = temp_dir.path().join("to_delete.txt");
        File::create(&file_path).unwrap();
        assert!(file_path.exists());

        // 完全削除を実行
        let result = manager.delete(&file_path, true);
        assert!(result.is_ok());

        // ファイルが削除されたことを確認
        assert!(!file_path.exists());
    }

    #[test]
    fn test_delete_permanent_directory() {
        let manager = FileManager::new();
        let temp_dir = tempdir().unwrap();

        // テストディレクトリを作成
        let dir_path = temp_dir.path().join("to_delete_dir");
        fs::create_dir(&dir_path).unwrap();
        let file_in_dir = dir_path.join("file.txt");
        File::create(&file_in_dir).unwrap();
        assert!(dir_path.exists());

        // ディレクトリの完全削除を実行
        let result = manager.delete(&dir_path, true);
        assert!(result.is_ok());

        // ディレクトリが削除されたことを確認
        assert!(!dir_path.exists());
    }

    #[test]
    fn test_delete_to_trash() {
        let manager = FileManager::new();
        let temp_dir = tempdir().unwrap();

        // テストファイルを作成
        let file_path = temp_dir.path().join("to_trash.txt");
        File::create(&file_path).unwrap();
        assert!(file_path.exists());

        // ゴミ箱へ移動
        let result = manager.delete(&file_path, false);
        assert!(result.is_ok());

        // ファイルが元の場所から削除されたことを確認
        assert!(!file_path.exists());
    }

    #[test]
    fn test_delete_nonexistent() {
        let manager = FileManager::new();
        let temp_dir = tempdir().unwrap();

        let file_path = temp_dir.path().join("nonexistent.txt");

        // 存在しないファイルの削除はエラーになる
        let result = manager.delete(&file_path, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("存在しません"));
    }

    #[test]
    fn test_rename() {
        let manager = FileManager::new();
        let temp_dir = tempdir().unwrap();

        // テストファイルを作成
        let old_path = temp_dir.path().join("old_name.txt");
        let mut file = File::create(&old_path).unwrap();
        writeln!(file, "リネームテスト").unwrap();

        // 名前を変更
        let result = manager.rename(&old_path, "new_name.txt");
        assert!(result.is_ok());

        // 古い名前のファイルが存在しないことを確認
        assert!(!old_path.exists());

        // 新しい名前のファイルが存在することを確認
        let new_path = temp_dir.path().join("new_name.txt");
        assert!(new_path.exists());

        // 内容が保持されていることを確認
        let content = fs::read_to_string(&new_path).unwrap();
        assert!(content.contains("リネームテスト"));
    }

    #[test]
    fn test_rename_nonexistent() {
        let manager = FileManager::new();
        let temp_dir = tempdir().unwrap();

        let file_path = temp_dir.path().join("nonexistent.txt");

        // 存在しないファイルのリネームはエラーになる
        let result = manager.rename(&file_path, "new_name.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("存在しません"));
    }

    #[test]
    fn test_rename_directory() {
        let manager = FileManager::new();
        let temp_dir = tempdir().unwrap();

        // テストディレクトリを作成
        let old_dir = temp_dir.path().join("old_dir");
        fs::create_dir(&old_dir).unwrap();

        // ディレクトリの名前を変更
        let result = manager.rename(&old_dir, "new_dir");
        assert!(result.is_ok());

        // 古い名前のディレクトリが存在しないことを確認
        assert!(!old_dir.exists());

        // 新しい名前のディレクトリが存在することを確認
        let new_dir = temp_dir.path().join("new_dir");
        assert!(new_dir.exists());
    }

    #[test]
    fn test_open_nonexistent() {
        let manager = FileManager::new();
        let temp_dir = tempdir().unwrap();

        let file_path = temp_dir.path().join("nonexistent.txt");

        // 存在しないファイルを開こうとするとエラーになる
        let result = manager.open(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("存在しません"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_open_existing_file() {
        let manager = FileManager::new();
        let temp_dir = tempdir().unwrap();

        // テストファイルを作成
        let file_path = temp_dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        // ファイルを開く（コマンドが起動できることを確認）
        let result = manager.open(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_messages_are_japanese() {
        let manager = FileManager::new();
        let temp_dir = tempdir().unwrap();

        // 存在しないパスでテスト
        let nonexistent = temp_dir.path().join("nonexistent.txt");

        // 各メソッドのエラーメッセージが日本語であることを確認
        let copy_err = manager.copy(&nonexistent, &temp_dir.path().join("dest.txt"));
        assert!(copy_err.is_err());
        let err_msg = copy_err.unwrap_err();
        assert!(err_msg.contains("存在しません"));

        let move_err = manager.move_file(&nonexistent, &temp_dir.path().join("dest.txt"));
        assert!(move_err.is_err());
        let err_msg = move_err.unwrap_err();
        assert!(err_msg.contains("存在しません"));

        let delete_err = manager.delete(&nonexistent, true);
        assert!(delete_err.is_err());
        let err_msg = delete_err.unwrap_err();
        assert!(err_msg.contains("存在しません"));

        let rename_err = manager.rename(&nonexistent, "new.txt");
        assert!(rename_err.is_err());
        let err_msg = rename_err.unwrap_err();
        assert!(err_msg.contains("存在しません"));

        let open_err = manager.open(&nonexistent);
        assert!(open_err.is_err());
        let err_msg = open_err.unwrap_err();
        assert!(err_msg.contains("存在しません"));
    }
}
