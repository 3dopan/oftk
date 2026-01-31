use crate::data::models::FileHistory;
use crate::data::storage;
use anyhow::Result;
use chrono::Utc;
use std::path::Path;

/// 履歴管理
#[derive(Debug, Clone)]
pub struct HistoryManager {
    history: Vec<FileHistory>,
    max_entries: usize,
}

impl HistoryManager {
    /// 新しい HistoryManager を作成
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            max_entries: 100,
        }
    }

    /// 履歴にエントリを追加
    pub fn add_entry(&mut self, path: &Path) {
        let path_buf = path.to_path_buf();
        let now = Utc::now();

        // 既存エントリを探す
        if let Some(entry) = self.history.iter_mut().find(|h| h.path == path_buf) {
            // 既存エントリがある場合は、アクセス日時を更新してカウントを増やす
            entry.accessed_at = now;
            entry.access_count += 1;
        } else {
            // 新規エントリを追加
            self.history.push(FileHistory {
                path: path_buf,
                accessed_at: now,
                access_count: 1,
            });
        }

        // 最大エントリ数を超えたら、古いものを削除（アクセス日時順）
        if self.history.len() > self.max_entries {
            // アクセス日時でソート（古い順）
            self.history.sort_by(|a, b| a.accessed_at.cmp(&b.accessed_at));
            // 古いものから削除
            self.history.drain(0..(self.history.len() - self.max_entries));
        }
    }

    /// 最近の履歴を取得（新しい順）
    pub fn get_recent(&self, limit: usize) -> Vec<FileHistory> {
        let mut sorted = self.history.clone();
        // 新しい順にソート
        sorted.sort_by(|a, b| b.accessed_at.cmp(&a.accessed_at));
        sorted.into_iter().take(limit).collect()
    }

    /// 履歴をクリア
    pub fn clear(&mut self) {
        self.history.clear();
    }

    /// 履歴の全エントリを取得
    pub fn get_all(&self) -> &[FileHistory] {
        &self.history
    }

    /// 履歴をファイルに保存
    pub fn save(&self) -> Result<()> {
        storage::save_history(&self.history)
    }

    /// ファイルから履歴を読み込み
    pub fn load(&mut self) -> Result<()> {
        self.history = storage::load_history()?;
        Ok(())
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::thread;
    use std::time::Duration;

    // テスト間で環境変数の設定が競合しないように、テストを直列化するためのロック
    static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_add_entry() {
        let mut manager = HistoryManager::new();
        let path = PathBuf::from("/path/to/file");

        manager.add_entry(&path);

        let all = manager.get_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].path, path);
        assert_eq!(all[0].access_count, 1);
    }

    #[test]
    fn test_add_entry_duplicate() {
        let mut manager = HistoryManager::new();
        let path = PathBuf::from("/path/to/file");

        manager.add_entry(&path);
        thread::sleep(Duration::from_millis(10));
        manager.add_entry(&path);

        let all = manager.get_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].path, path);
        assert_eq!(all[0].access_count, 2);
    }

    #[test]
    fn test_add_entry_multiple() {
        let mut manager = HistoryManager::new();

        for i in 1..=5 {
            let path = PathBuf::from(format!("/path/to/file{}", i));
            manager.add_entry(&path);
        }

        let all = manager.get_all();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_max_entries() {
        let mut manager = HistoryManager::new();

        // 101個のエントリを追加（max_entries = 100）
        for i in 1..=101 {
            let path = PathBuf::from(format!("/path/to/file{}", i));
            manager.add_entry(&path);
            thread::sleep(Duration::from_millis(1));
        }

        // 最大100個に制限されているはず
        let all = manager.get_all();
        assert_eq!(all.len(), 100);

        // 最も古いエントリ（file1）が削除されているはず
        assert!(!all.iter().any(|h| h.path == PathBuf::from("/path/to/file1")));
    }

    #[test]
    fn test_get_recent() {
        let mut manager = HistoryManager::new();

        // 10個のエントリを追加
        for i in 1..=10 {
            let path = PathBuf::from(format!("/path/to/file{}", i));
            manager.add_entry(&path);
            thread::sleep(Duration::from_millis(1));
        }

        // 最近の5件を取得
        let recent = manager.get_recent(5);
        assert_eq!(recent.len(), 5);

        // 新しい順になっているはず（file10, file9, file8, file7, file6）
        assert_eq!(recent[0].path, PathBuf::from("/path/to/file10"));
        assert_eq!(recent[1].path, PathBuf::from("/path/to/file9"));
        assert_eq!(recent[2].path, PathBuf::from("/path/to/file8"));
        assert_eq!(recent[3].path, PathBuf::from("/path/to/file7"));
        assert_eq!(recent[4].path, PathBuf::from("/path/to/file6"));
    }

    #[test]
    fn test_get_recent_less_than_limit() {
        let mut manager = HistoryManager::new();

        // 3個のエントリを追加
        for i in 1..=3 {
            let path = PathBuf::from(format!("/path/to/file{}", i));
            manager.add_entry(&path);
        }

        // 10件要求しても、3件しか返らない
        let recent = manager.get_recent(10);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_clear() {
        let mut manager = HistoryManager::new();

        // エントリを追加
        for i in 1..=5 {
            let path = PathBuf::from(format!("/path/to/file{}", i));
            manager.add_entry(&path);
        }

        assert_eq!(manager.get_all().len(), 5);

        // クリア
        manager.clear();
        assert_eq!(manager.get_all().len(), 0);
    }

    #[test]
    fn test_save_and_load() {
        use std::env;
        use std::fs;

        let _lock = TEST_ENV_LOCK.lock().unwrap();

        let temp_dir = env::temp_dir().join(format!("ofkt_history_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        let original_config_home = env::var("XDG_CONFIG_HOME").ok();

        struct EnvGuard {
            original: Option<String>,
            temp_dir: PathBuf,
        }

        impl Drop for EnvGuard {
            fn drop(&mut self) {
                if let Some(original) = &self.original {
                    env::set_var("XDG_CONFIG_HOME", original);
                } else {
                    env::remove_var("XDG_CONFIG_HOME");
                }
                fs::remove_dir_all(&self.temp_dir).ok();
            }
        }

        let _guard = EnvGuard {
            original: original_config_home,
            temp_dir: temp_dir.clone(),
        };

        env::set_var("XDG_CONFIG_HOME", &temp_dir);

        // エントリを追加
        let mut manager = HistoryManager::new();
        for i in 1..=5 {
            let path = PathBuf::from(format!("/path/to/file{}", i));
            manager.add_entry(&path);
            thread::sleep(Duration::from_millis(1));
        }

        // 保存
        let save_result = manager.save();
        assert!(save_result.is_ok());

        // 新しいマネージャーで読み込み
        let mut new_manager = HistoryManager::new();
        let load_result = new_manager.load();
        assert!(load_result.is_ok());

        // データが一致することを確認
        assert_eq!(new_manager.get_all().len(), 5);
        assert_eq!(new_manager.get_all()[0].path, PathBuf::from("/path/to/file1"));
        assert_eq!(new_manager.get_all()[4].path, PathBuf::from("/path/to/file5"));
    }

    #[test]
    fn test_load_empty() {
        use std::env;
        use std::fs;

        let _lock = TEST_ENV_LOCK.lock().unwrap();

        let temp_dir = env::temp_dir().join(format!("ofkt_history_empty_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        let original_config_home = env::var("XDG_CONFIG_HOME").ok();

        struct EnvGuard {
            original: Option<String>,
            temp_dir: PathBuf,
        }

        impl Drop for EnvGuard {
            fn drop(&mut self) {
                if let Some(original) = &self.original {
                    env::set_var("XDG_CONFIG_HOME", original);
                } else {
                    env::remove_var("XDG_CONFIG_HOME");
                }
                fs::remove_dir_all(&self.temp_dir).ok();
            }
        }

        let _guard = EnvGuard {
            original: original_config_home,
            temp_dir: temp_dir.clone(),
        };

        env::set_var("XDG_CONFIG_HOME", &temp_dir);

        // 履歴ファイルが存在しない状態でロード
        let mut manager = HistoryManager::new();
        let load_result = manager.load();
        assert!(load_result.is_ok());
        assert_eq!(manager.get_all().len(), 0);
    }

    #[test]
    fn test_access_count_increments() {
        let mut manager = HistoryManager::new();
        let path = PathBuf::from("/path/to/file");

        // 同じファイルを3回アクセス
        for _ in 0..3 {
            manager.add_entry(&path);
            thread::sleep(Duration::from_millis(1));
        }

        let all = manager.get_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].access_count, 3);
    }

    #[test]
    fn test_accessed_at_updates() {
        let mut manager = HistoryManager::new();
        let path = PathBuf::from("/path/to/file");

        manager.add_entry(&path);
        let first_access = manager.get_all()[0].accessed_at;

        thread::sleep(Duration::from_millis(10));

        manager.add_entry(&path);
        let second_access = manager.get_all()[0].accessed_at;

        // アクセス日時が更新されているはず
        assert!(second_access > first_access);
    }
}
