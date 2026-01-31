//! ディレクトリブラウザモジュール
//!
//! ファイルシステムの動的閲覧機能を提供します。

use std::path::{Path, PathBuf};
use std::io;
use crate::data::models::DirectoryEntry;

/// ディレクトリブラウザ
///
/// ファイルシステムを閲覧し、ナビゲーション履歴を管理します。
#[derive(Debug, Clone)]
pub struct DirectoryBrowser {
    /// 現在表示しているディレクトリのパス
    current_path: PathBuf,

    /// 現在のディレクトリのエントリ一覧
    entries: Vec<DirectoryEntry>,

    /// ナビゲーション履歴（戻る/進む用）
    history: Vec<PathBuf>,

    /// 履歴内の現在位置
    history_index: usize,

    /// 隠しファイル/フォルダを表示するか
    show_hidden: bool,
}

impl DirectoryBrowser {
    /// 指定パスでDirectoryBrowserを作成
    ///
    /// # Arguments
    ///
    /// * `path` - 開くディレクトリのパス
    ///
    /// # Returns
    ///
    /// * `Ok(DirectoryBrowser)` - 成功時
    /// * `Err(io::Error)` - パスが存在しない、またはディレクトリでない場合
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    /// use ofkt::core::directory_browser::DirectoryBrowser;
    ///
    /// let browser = DirectoryBrowser::new(PathBuf::from("C:\\Users")).unwrap();
    /// ```
    pub fn new(path: PathBuf) -> io::Result<Self> {
        // パスが存在し、ディレクトリであることを確認
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Path does not exist: {}", path.display()),
            ));
        }

        if !path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Path is not a directory: {}", path.display()),
            ));
        }

        let mut browser = Self {
            current_path: path.clone(),
            entries: Vec::new(),
            history: vec![path],
            history_index: 0,
            show_hidden: false,
        };

        // 初期エントリを読み込み
        browser.load_entries()?;

        Ok(browser)
    }

    /// 現在のパスを取得
    ///
    /// # Returns
    ///
    /// 現在表示しているディレクトリのパス
    pub fn current_path(&self) -> &Path {
        &self.current_path
    }

    /// 現在のエントリ一覧を取得
    ///
    /// # Returns
    ///
    /// 現在のディレクトリ内のエントリ一覧
    pub fn entries(&self) -> &[DirectoryEntry] {
        &self.entries
    }

    /// 指定パスに移動
    ///
    /// # Arguments
    ///
    /// * `path` - 移動先のディレクトリパス
    ///
    /// # Returns
    ///
    /// * `Ok(())` - 成功時
    /// * `Err(io::Error)` - パスが存在しない、またはディレクトリでない場合
    pub fn navigate_to(&mut self, path: PathBuf) -> io::Result<()> {
        // パスが存在し、ディレクトリであることを確認
        if !path.exists() {
            if is_wsl_path(&path) {
                log::warn!("WSLパスが見つかりません: {}", path.display());
            }
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Path does not exist: {}", path.display()),
            ));
        }

        if !path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Path is not a directory: {}", path.display()),
            ));
        }

        // 現在のパスを更新
        self.current_path = path.clone();

        // 履歴を更新（現在位置より後ろの履歴は削除）
        self.history.truncate(self.history_index + 1);
        self.history.push(path);
        self.history_index = self.history.len() - 1;

        // エントリを読み込み
        self.load_entries()?;

        Ok(())
    }

    /// 親ディレクトリに移動
    ///
    /// # Returns
    ///
    /// * `Ok(())` - 成功時
    /// * `Err(io::Error)` - 親ディレクトリが存在しない場合
    pub fn parent(&mut self) -> io::Result<()> {
        if let Some(parent) = self.current_path.parent() {
            let parent = parent.to_path_buf();
            self.navigate_to(parent)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No parent directory exists",
            ))
        }
    }

    /// 戻れるかどうかを確認
    ///
    /// # Returns
    ///
    /// 履歴で戻れる場合はtrue
    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    /// 進めるかどうかを確認
    ///
    /// # Returns
    ///
    /// 履歴で進める場合はtrue
    pub fn can_go_forward(&self) -> bool {
        self.history_index < self.history.len() - 1
    }

    /// 履歴で戻る
    ///
    /// # Returns
    ///
    /// * `Ok(())` - 成功時
    /// * `Err(io::Error)` - 戻れる履歴がない場合、またはディレクトリへのアクセスに失敗した場合
    pub fn go_back(&mut self) -> io::Result<()> {
        if !self.can_go_back() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot go back: no previous history",
            ));
        }

        self.history_index -= 1;
        self.current_path = self.history[self.history_index].clone();
        self.load_entries()?;

        Ok(())
    }

    /// 履歴で進む
    ///
    /// # Returns
    ///
    /// * `Ok(())` - 成功時
    /// * `Err(io::Error)` - 進める履歴がない場合、またはディレクトリへのアクセスに失敗した場合
    pub fn go_forward(&mut self) -> io::Result<()> {
        if !self.can_go_forward() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot go forward: no forward history",
            ));
        }

        self.history_index += 1;
        self.current_path = self.history[self.history_index].clone();
        self.load_entries()?;

        Ok(())
    }

    /// 現在のディレクトリを再読み込み
    ///
    /// # Returns
    ///
    /// * `Ok(())` - 成功時
    /// * `Err(io::Error)` - ディレクトリの読み込みに失敗した場合
    pub fn reload(&mut self) -> io::Result<()> {
        self.load_entries()
    }

    /// 隠しファイル/フォルダの表示設定を変更
    ///
    /// # Arguments
    ///
    /// * `show` - trueの場合、隠しファイル/フォルダを表示
    pub fn set_show_hidden(&mut self, show: bool) {
        self.show_hidden = show;
    }

    /// 内部メソッド: エントリを読み込む
    ///
    /// 現在のパスからディレクトリエントリを読み込み、ソートします。
    /// 隠しファイルの表示設定に基づいてフィルタリングも行います。
    ///
    /// # Returns
    ///
    /// * `Ok(())` - 成功時
    /// * `Err(io::Error)` - ディレクトリの読み込みに失敗した場合
    fn load_entries(&mut self) -> io::Result<()> {
        // WSLパスの場合の特別処理
        if is_wsl_path(&self.current_path) {
            log::info!("WSLパスを読み込み: {}", self.current_path.display());
        }

        let mut entries = Vec::new();

        // ディレクトリを読み込む
        let dir_result = std::fs::read_dir(&self.current_path);
        if let Err(e) = &dir_result {
            if is_wsl_path(&self.current_path) {
                log::error!("WSLパスの読み込みエラー: {} - {}", self.current_path.display(), e);
            }
        }

        for entry in dir_result? {
            let entry = entry?;
            let path = entry.path();

            // DirectoryEntryを作成
            match DirectoryEntry::from_path(path) {
                Ok(dir_entry) => {
                    // 隠しファイルのフィルタリング
                    if !self.show_hidden && dir_entry.is_hidden {
                        continue;
                    }
                    entries.push(dir_entry);
                }
                Err(e) => {
                    // アクセス権限エラーなどは無視して続行
                    eprintln!("Warning: Failed to read entry: {}", e);
                }
            }
        }

        // エントリをソート（ディレクトリ優先、その後名前順）
        entries.sort_by(|a, b| {
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        self.entries = entries;

        Ok(())
    }
}

/// WSLパスかどうかを判定
fn is_wsl_path(path: &Path) -> bool {
    path.to_string_lossy().starts_with(r"\\wsl")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::env;

    /// テスト用の一時ディレクトリを作成するヘルパー
    fn create_test_dir() -> PathBuf {
        let temp_dir = env::temp_dir().join(format!("ofkt_test_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();
        temp_dir
    }

    /// テスト用の一時ディレクトリを削除するヘルパー
    fn cleanup_test_dir(path: &PathBuf) {
        fs::remove_dir_all(path).ok();
    }

    #[test]
    fn test_new_with_valid_directory() {
        let test_dir = create_test_dir();
        let browser = DirectoryBrowser::new(test_dir.clone());
        assert!(browser.is_ok());

        let browser = browser.unwrap();
        assert_eq!(browser.current_path(), test_dir.as_path());
        assert_eq!(browser.entries().len(), 0);

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_new_with_nonexistent_path() {
        let nonexistent = PathBuf::from("C:\\nonexistent_path_12345");
        let result = DirectoryBrowser::new(nonexistent);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn test_new_with_file_path() {
        let test_dir = create_test_dir();
        let test_file = test_dir.join("test_file.txt");
        fs::write(&test_file, "test content").unwrap();

        let result = DirectoryBrowser::new(test_file);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_navigate_to_subdirectory() {
        let test_dir = create_test_dir();
        let subdir = test_dir.join("subdir");
        fs::create_dir(&subdir).unwrap();

        let mut browser = DirectoryBrowser::new(test_dir.clone()).unwrap();
        let result = browser.navigate_to(subdir.clone());
        assert!(result.is_ok());
        assert_eq!(browser.current_path(), subdir.as_path());

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_navigate_to_invalid_path() {
        let test_dir = create_test_dir();
        let mut browser = DirectoryBrowser::new(test_dir.clone()).unwrap();

        let invalid_path = PathBuf::from("C:\\invalid_path_99999");
        let result = browser.navigate_to(invalid_path);
        assert!(result.is_err());

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_parent_navigation() {
        let test_dir = create_test_dir();
        let subdir = test_dir.join("subdir");
        fs::create_dir(&subdir).unwrap();

        let mut browser = DirectoryBrowser::new(subdir.clone()).unwrap();
        let result = browser.parent();
        assert!(result.is_ok());
        assert_eq!(browser.current_path(), test_dir.as_path());

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_history_navigation() {
        let test_dir = create_test_dir();
        let subdir1 = test_dir.join("subdir1");
        let subdir2 = test_dir.join("subdir2");
        fs::create_dir(&subdir1).unwrap();
        fs::create_dir(&subdir2).unwrap();

        let mut browser = DirectoryBrowser::new(test_dir.clone()).unwrap();

        // 初期状態では戻れない、進めない
        assert!(!browser.can_go_back());
        assert!(!browser.can_go_forward());

        // subdir1に移動
        browser.navigate_to(subdir1.clone()).unwrap();
        assert!(browser.can_go_back());
        assert!(!browser.can_go_forward());

        // subdir2に移動
        browser.navigate_to(subdir2.clone()).unwrap();
        assert!(browser.can_go_back());
        assert!(!browser.can_go_forward());

        // 戻る
        browser.go_back().unwrap();
        assert_eq!(browser.current_path(), subdir1.as_path());
        assert!(browser.can_go_back());
        assert!(browser.can_go_forward());

        // さらに戻る
        browser.go_back().unwrap();
        assert_eq!(browser.current_path(), test_dir.as_path());
        assert!(!browser.can_go_back());
        assert!(browser.can_go_forward());

        // 進む
        browser.go_forward().unwrap();
        assert_eq!(browser.current_path(), subdir1.as_path());
        assert!(browser.can_go_back());
        assert!(browser.can_go_forward());

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_go_back_without_history() {
        let test_dir = create_test_dir();
        let mut browser = DirectoryBrowser::new(test_dir.clone()).unwrap();

        let result = browser.go_back();
        assert!(result.is_err());

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_go_forward_without_history() {
        let test_dir = create_test_dir();
        let mut browser = DirectoryBrowser::new(test_dir.clone()).unwrap();

        let result = browser.go_forward();
        assert!(result.is_err());

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_reload() {
        let test_dir = create_test_dir();
        let mut browser = DirectoryBrowser::new(test_dir.clone()).unwrap();

        // 初期状態は空
        assert_eq!(browser.entries().len(), 0);

        // ファイルを追加
        fs::write(test_dir.join("file1.txt"), "content").unwrap();

        // リロード
        browser.reload().unwrap();
        assert_eq!(browser.entries().len(), 1);

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_show_hidden_files() {
        let test_dir = create_test_dir();

        // 通常ファイルを作成
        fs::write(test_dir.join("visible.txt"), "content").unwrap();

        // 隠しファイルを作成（Windowsでは属性を設定する必要があるが、簡易テストとして名前のみで判定）
        #[cfg(not(target_os = "windows"))]
        fs::write(test_dir.join(".hidden.txt"), "content").unwrap();

        let mut browser = DirectoryBrowser::new(test_dir.clone()).unwrap();

        // デフォルトでは隠しファイルは表示されない
        #[cfg(target_os = "windows")]
        assert_eq!(browser.entries().len(), 1);

        #[cfg(not(target_os = "windows"))]
        assert_eq!(browser.entries().len(), 1);

        // 隠しファイルを表示
        browser.set_show_hidden(true);
        browser.reload().unwrap();

        #[cfg(target_os = "windows")]
        assert_eq!(browser.entries().len(), 1);

        #[cfg(not(target_os = "windows"))]
        assert_eq!(browser.entries().len(), 2);

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_entries_sorted() {
        let test_dir = create_test_dir();

        // ディレクトリとファイルを作成
        fs::create_dir(test_dir.join("z_dir")).unwrap();
        fs::create_dir(test_dir.join("a_dir")).unwrap();
        fs::write(test_dir.join("z_file.txt"), "content").unwrap();
        fs::write(test_dir.join("a_file.txt"), "content").unwrap();

        let browser = DirectoryBrowser::new(test_dir.clone()).unwrap();
        let entries = browser.entries();

        // ディレクトリが先に来る
        assert_eq!(entries.len(), 4);
        assert!(entries[0].is_directory);
        assert!(entries[1].is_directory);
        assert!(entries[2].is_file());
        assert!(entries[3].is_file());

        // ディレクトリは名前順
        assert_eq!(entries[0].name, "a_dir");
        assert_eq!(entries[1].name, "z_dir");

        // ファイルも名前順
        assert_eq!(entries[2].name, "a_file.txt");
        assert_eq!(entries[3].name, "z_file.txt");

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_history_truncation_on_new_navigation() {
        let test_dir = create_test_dir();
        let subdir1 = test_dir.join("subdir1");
        let subdir2 = test_dir.join("subdir2");
        let subdir3 = test_dir.join("subdir3");
        fs::create_dir(&subdir1).unwrap();
        fs::create_dir(&subdir2).unwrap();
        fs::create_dir(&subdir3).unwrap();

        let mut browser = DirectoryBrowser::new(test_dir.clone()).unwrap();

        // subdir1 -> subdir2に移動
        browser.navigate_to(subdir1.clone()).unwrap();
        browser.navigate_to(subdir2.clone()).unwrap();

        // 戻る
        browser.go_back().unwrap();
        assert!(browser.can_go_forward());

        // 新しいパスに移動すると、進む履歴が削除される
        browser.navigate_to(subdir3.clone()).unwrap();
        assert!(!browser.can_go_forward());
        assert_eq!(browser.current_path(), subdir3.as_path());

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_current_path_getter() {
        let test_dir = create_test_dir();
        let browser = DirectoryBrowser::new(test_dir.clone()).unwrap();

        assert_eq!(browser.current_path(), test_dir.as_path());

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_is_wsl_path() {
        use std::path::Path;

        // WSLパスのテスト
        assert!(is_wsl_path(Path::new(r"\\wsl$\Ubuntu\home\user")));
        assert!(is_wsl_path(Path::new(r"\\wsl.localhost\Ubuntu\home\user")));
        assert!(is_wsl_path(Path::new(r"\\wsl\Ubuntu")));

        // 非WSLパスのテスト
        assert!(!is_wsl_path(Path::new(r"C:\Users\test")));
        assert!(!is_wsl_path(Path::new(r"\\network\share")));
        assert!(!is_wsl_path(Path::new(r"D:\data")));
    }

    #[test]
    fn test_entries_getter() {
        let test_dir = create_test_dir();
        fs::write(test_dir.join("file.txt"), "content").unwrap();

        let browser = DirectoryBrowser::new(test_dir.clone()).unwrap();
        let entries = browser.entries();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "file.txt");

        cleanup_test_dir(&test_dir);
    }
}
