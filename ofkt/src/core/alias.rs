use crate::data::models::FileAlias;
use crate::data::storage;
use anyhow::Result;
use chrono::Utc;
use std::path::PathBuf;
use uuid::Uuid;

/// エイリアス管理
#[derive(Debug, Clone)]
pub struct AliasManager {
    aliases: Vec<FileAlias>,
}

impl AliasManager {
    /// 新しい AliasManager を作成
    pub fn new() -> Self {
        Self {
            aliases: Vec::new(),
        }
    }

    /// エイリアスを追加
    pub fn add_alias(
        &mut self,
        alias: String,
        path: PathBuf,
        tags: Vec<String>,
        color: Option<String>,
        is_favorite: bool,
    ) -> Result<(), String> {
        // 重複チェック
        if self.aliases.iter().any(|a| a.alias == alias) {
            return Err(format!("エイリアス '{}' は既に存在します", alias));
        }

        // UUID生成
        let id = Uuid::new_v4().to_string();

        // タイムスタンプ生成
        let now = Utc::now();

        // FileAlias作成
        let file_alias = FileAlias {
            id,
            alias,
            path,
            tags,
            color,
            created_at: now,
            last_accessed: now,
            is_favorite,
        };

        // リストに追加
        self.aliases.push(file_alias);

        Ok(())
    }

    /// エイリアス一覧を取得
    pub fn get_aliases(&self) -> &[FileAlias] {
        &self.aliases
    }

    /// IDでエイリアスを削除
    pub fn remove_alias_by_id(&mut self, id: &str) -> Result<(), String> {
        let index = self
            .aliases
            .iter()
            .position(|a| a.id == id)
            .ok_or_else(|| format!("エイリアスID '{}' は存在しません", id))?;

        self.aliases.remove(index);
        Ok(())
    }

    /// 名前でエイリアスを削除
    pub fn remove_alias_by_name(&mut self, alias: &str) -> Result<(), String> {
        let index = self
            .aliases
            .iter()
            .position(|a| a.alias == alias)
            .ok_or_else(|| format!("エイリアス '{}' は存在しません", alias))?;

        self.aliases.remove(index);
        Ok(())
    }

    /// エイリアスを更新
    pub fn update_alias(
        &mut self,
        id: &str,
        alias: Option<String>,
        path: Option<PathBuf>,
        tags: Option<Vec<String>>,
        color: Option<Option<String>>,
        is_favorite: Option<bool>,
    ) -> Result<(), String> {
        let file_alias = self
            .aliases
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| format!("エイリアスID '{}' は存在しません", id))?;

        // Option値の更新
        if let Some(alias_val) = alias {
            file_alias.alias = alias_val;
        }
        if let Some(path_val) = path {
            file_alias.path = path_val;
        }
        if let Some(tags_val) = tags {
            file_alias.tags = tags_val;
        }
        if let Some(color_val) = color {
            file_alias.color = color_val;
        }
        if let Some(is_favorite_val) = is_favorite {
            file_alias.is_favorite = is_favorite_val;
        }

        Ok(())
    }

    /// エイリアスリストをファイルに保存
    pub fn save(&self) -> Result<()> {
        storage::save_aliases(&self.aliases)
    }

    /// ファイルからエイリアスリストを読み込み
    pub fn load(&mut self) -> Result<()> {
        self.aliases = storage::load_aliases()?;
        Ok(())
    }

    /// お気に入りの切り替え
    pub fn toggle_favorite(&mut self, id: &str) -> Result<(), String> {
        let alias = self.aliases
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| format!("エイリアスID '{}' は存在しません", id))?;

        alias.is_favorite = !alias.is_favorite;
        Ok(())
    }

    /// お気に入り一覧を取得
    pub fn get_favorites(&self) -> Vec<&FileAlias> {
        self.aliases
            .iter()
            .filter(|a| a.is_favorite)
            .collect()
    }
}

impl Default for AliasManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Mutex;

    // テスト間で環境変数の設定が競合しないように、テストを直列化するためのロック
    static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_add_alias() {
        let mut manager = AliasManager::new();
        let result = manager.add_alias(
            "test".to_string(),
            PathBuf::from("/path/to/file"),
            vec![],
            None,
            false,
        );

        assert!(result.is_ok());
        assert_eq!(manager.get_aliases().len(), 1);

        let alias = &manager.get_aliases()[0];
        assert_eq!(alias.alias, "test");
        assert_eq!(alias.path, PathBuf::from("/path/to/file"));
        assert_eq!(alias.tags, Vec::<String>::new());
        assert_eq!(alias.color, None);
        assert_eq!(alias.is_favorite, false);
    }

    #[test]
    fn test_add_duplicate_alias() {
        let mut manager = AliasManager::new();

        // 最初のエイリアス追加
        let result1 = manager.add_alias(
            "duplicate".to_string(),
            PathBuf::from("/path/to/file1"),
            vec![],
            None,
            false,
        );
        assert!(result1.is_ok());

        // 同じ名前で重複追加を試みる
        let result2 = manager.add_alias(
            "duplicate".to_string(),
            PathBuf::from("/path/to/file2"),
            vec![],
            None,
            false,
        );
        assert!(result2.is_err());
        assert_eq!(
            result2.unwrap_err(),
            "エイリアス 'duplicate' は既に存在します"
        );

        // エイリアスが1つだけ存在することを確認
        assert_eq!(manager.get_aliases().len(), 1);
    }

    #[test]
    fn test_add_alias_generates_uuid() {
        let mut manager = AliasManager::new();

        // 複数のエイリアスを追加
        manager
            .add_alias(
                "alias1".to_string(),
                PathBuf::from("/path/to/file1"),
                vec![],
                None,
                false,
            )
            .unwrap();

        manager
            .add_alias(
                "alias2".to_string(),
                PathBuf::from("/path/to/file2"),
                vec![],
                None,
                false,
            )
            .unwrap();

        let aliases = manager.get_aliases();
        assert_eq!(aliases.len(), 2);

        // UUID が生成されていることを確認
        assert!(!aliases[0].id.is_empty());
        assert!(!aliases[1].id.is_empty());

        // UUID が異なることを確認
        assert_ne!(aliases[0].id, aliases[1].id);

        // UUID の形式を確認（UUIDとしてパース可能か）
        assert!(Uuid::parse_str(&aliases[0].id).is_ok());
        assert!(Uuid::parse_str(&aliases[1].id).is_ok());
    }

    #[test]
    fn test_add_alias_with_tags_and_color() {
        let mut manager = AliasManager::new();
        let result = manager.add_alias(
            "tagged".to_string(),
            PathBuf::from("/path/to/file"),
            vec!["important".to_string(), "work".to_string()],
            Some("#FF0000".to_string()),
            true,
        );

        assert!(result.is_ok());

        let alias = &manager.get_aliases()[0];
        assert_eq!(alias.tags, vec!["important", "work"]);
        assert_eq!(alias.color, Some("#FF0000".to_string()));
        assert_eq!(alias.is_favorite, true);
    }

    #[test]
    fn test_add_alias_timestamps() {
        let mut manager = AliasManager::new();
        let before = Utc::now();

        manager
            .add_alias(
                "timestamp_test".to_string(),
                PathBuf::from("/path/to/file"),
                vec![],
                None,
                false,
            )
            .unwrap();

        let after = Utc::now();
        let alias = &manager.get_aliases()[0];

        // created_at と last_accessed が同じであることを確認
        assert_eq!(alias.created_at, alias.last_accessed);

        // タイムスタンプが追加前後の間にあることを確認
        assert!(alias.created_at >= before);
        assert!(alias.created_at <= after);
    }

    #[test]
    fn test_multiple_aliases() {
        let mut manager = AliasManager::new();

        // 複数のエイリアスを追加
        for i in 1..=5 {
            let result = manager.add_alias(
                format!("alias{}", i),
                PathBuf::from(format!("/path/to/file{}", i)),
                vec![],
                None,
                false,
            );
            assert!(result.is_ok());
        }

        // 5つのエイリアスが追加されていることを確認
        assert_eq!(manager.get_aliases().len(), 5);

        // 各エイリアスが正しく設定されていることを確認
        for (i, alias) in manager.get_aliases().iter().enumerate() {
            assert_eq!(alias.alias, format!("alias{}", i + 1));
            assert_eq!(alias.path, PathBuf::from(format!("/path/to/file{}", i + 1)));
        }
    }

    #[test]
    fn test_remove_alias_by_id() {
        let mut manager = AliasManager::new();

        // エイリアスを追加
        manager
            .add_alias(
                "test1".to_string(),
                PathBuf::from("/path/to/file1"),
                vec![],
                None,
                false,
            )
            .unwrap();

        manager
            .add_alias(
                "test2".to_string(),
                PathBuf::from("/path/to/file2"),
                vec![],
                None,
                false,
            )
            .unwrap();

        // 2つのエイリアスが存在することを確認
        assert_eq!(manager.get_aliases().len(), 2);

        // 最初のエイリアスのIDを取得
        let id = manager.get_aliases()[0].id.clone();

        // IDで削除
        let result = manager.remove_alias_by_id(&id);
        assert!(result.is_ok());

        // 1つだけ残っていることを確認
        assert_eq!(manager.get_aliases().len(), 1);

        // 残っているエイリアスが "test2" であることを確認
        assert_eq!(manager.get_aliases()[0].alias, "test2");
    }

    #[test]
    fn test_remove_alias_by_name() {
        let mut manager = AliasManager::new();

        // エイリアスを追加
        manager
            .add_alias(
                "test1".to_string(),
                PathBuf::from("/path/to/file1"),
                vec![],
                None,
                false,
            )
            .unwrap();

        manager
            .add_alias(
                "test2".to_string(),
                PathBuf::from("/path/to/file2"),
                vec![],
                None,
                false,
            )
            .unwrap();

        // 2つのエイリアスが存在することを確認
        assert_eq!(manager.get_aliases().len(), 2);

        // 名前で削除
        let result = manager.remove_alias_by_name("test1");
        assert!(result.is_ok());

        // 1つだけ残っていることを確認
        assert_eq!(manager.get_aliases().len(), 1);

        // 残っているエイリアスが "test2" であることを確認
        assert_eq!(manager.get_aliases()[0].alias, "test2");
    }

    #[test]
    fn test_remove_nonexistent_alias() {
        let mut manager = AliasManager::new();

        // エイリアスを追加
        manager
            .add_alias(
                "test".to_string(),
                PathBuf::from("/path/to/file"),
                vec![],
                None,
                false,
            )
            .unwrap();

        // 存在しないIDで削除を試みる
        let result = manager.remove_alias_by_id("nonexistent-id");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "エイリアスID 'nonexistent-id' は存在しません"
        );

        // 存在しない名前で削除を試みる
        let result = manager.remove_alias_by_name("nonexistent");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "エイリアス 'nonexistent' は存在しません"
        );

        // エイリアスが削除されていないことを確認
        assert_eq!(manager.get_aliases().len(), 1);
    }

    #[test]
    fn test_update_alias() {
        let mut manager = AliasManager::new();

        // エイリアスを追加
        manager
            .add_alias(
                "original".to_string(),
                PathBuf::from("/path/to/original"),
                vec!["tag1".to_string()],
                Some("#FF0000".to_string()),
                false,
            )
            .unwrap();

        let id = manager.get_aliases()[0].id.clone();

        // すべてのフィールドを更新
        let result = manager.update_alias(
            &id,
            Some("updated".to_string()),
            Some(PathBuf::from("/path/to/updated")),
            Some(vec!["tag2".to_string(), "tag3".to_string()]),
            Some(Some("#00FF00".to_string())),
            Some(true),
        );

        assert!(result.is_ok());

        let alias = &manager.get_aliases()[0];
        assert_eq!(alias.alias, "updated");
        assert_eq!(alias.path, PathBuf::from("/path/to/updated"));
        assert_eq!(alias.tags, vec!["tag2", "tag3"]);
        assert_eq!(alias.color, Some("#00FF00".to_string()));
        assert_eq!(alias.is_favorite, true);
    }

    #[test]
    fn test_update_alias_partial() {
        let mut manager = AliasManager::new();

        // エイリアスを追加
        manager
            .add_alias(
                "original".to_string(),
                PathBuf::from("/path/to/original"),
                vec!["tag1".to_string()],
                Some("#FF0000".to_string()),
                false,
            )
            .unwrap();

        let id = manager.get_aliases()[0].id.clone();
        let original_path = manager.get_aliases()[0].path.clone();
        let original_color = manager.get_aliases()[0].color.clone();

        // エイリアス名とお気に入りのみ更新
        let result = manager.update_alias(
            &id,
            Some("partial_update".to_string()),
            None,
            None,
            None,
            Some(true),
        );

        assert!(result.is_ok());

        let alias = &manager.get_aliases()[0];
        assert_eq!(alias.alias, "partial_update");
        assert_eq!(alias.path, original_path); // 変更されていない
        assert_eq!(alias.tags, vec!["tag1"]); // 変更されていない
        assert_eq!(alias.color, original_color); // 変更されていない
        assert_eq!(alias.is_favorite, true); // 更新されている
    }

    #[test]
    fn test_update_nonexistent_alias() {
        let mut manager = AliasManager::new();

        // エイリアスを追加
        manager
            .add_alias(
                "test".to_string(),
                PathBuf::from("/path/to/file"),
                vec![],
                None,
                false,
            )
            .unwrap();

        // 存在しないIDで更新を試みる
        let result = manager.update_alias(
            "nonexistent-id",
            Some("updated".to_string()),
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "エイリアスID 'nonexistent-id' は存在しません"
        );

        // 元のエイリアスが変更されていないことを確認
        let alias = &manager.get_aliases()[0];
        assert_eq!(alias.alias, "test");
    }

    #[test]
    fn test_update_alias_clear_color() {
        let mut manager = AliasManager::new();

        // 色付きのエイリアスを追加
        manager
            .add_alias(
                "colored".to_string(),
                PathBuf::from("/path/to/file"),
                vec![],
                Some("#FF0000".to_string()),
                false,
            )
            .unwrap();

        let id = manager.get_aliases()[0].id.clone();

        // 色をクリア (None に設定)
        let result = manager.update_alias(&id, None, None, None, Some(None), None);

        assert!(result.is_ok());

        let alias = &manager.get_aliases()[0];
        assert_eq!(alias.color, None);
    }

    #[test]
    fn test_update_alias_timestamps_unchanged() {
        let mut manager = AliasManager::new();

        // エイリアスを追加
        manager
            .add_alias(
                "timestamp_test".to_string(),
                PathBuf::from("/path/to/file"),
                vec![],
                None,
                false,
            )
            .unwrap();

        let id = manager.get_aliases()[0].id.clone();
        let original_created_at = manager.get_aliases()[0].created_at;
        let original_last_accessed = manager.get_aliases()[0].last_accessed;

        // エイリアスを更新
        manager
            .update_alias(&id, Some("updated".to_string()), None, None, None, None)
            .unwrap();

        let alias = &manager.get_aliases()[0];

        // タイムスタンプが変更されていないことを確認
        assert_eq!(alias.created_at, original_created_at);
        assert_eq!(alias.last_accessed, original_last_accessed);
    }

    #[test]
    fn test_save_and_load() {
        use std::env;
        use std::fs;

        // 環境変数の競合を防ぐためにロックを取得
        let _lock = TEST_ENV_LOCK.lock().unwrap();

        // テスト用の一時ディレクトリを設定
        let temp_dir = env::temp_dir().join(format!("ofkt_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // 元の設定ディレクトリを保存
        let original_config_home = env::var("XDG_CONFIG_HOME").ok();

        // スコープガード: テスト終了時に必ず環境変数を復元
        struct EnvGuard {
            original: Option<String>,
            temp_dir: PathBuf,
        }

        impl Drop for EnvGuard {
            fn drop(&mut self) {
                // 元の環境変数に戻す
                if let Some(original) = &self.original {
                    env::set_var("XDG_CONFIG_HOME", original);
                } else {
                    env::remove_var("XDG_CONFIG_HOME");
                }
                // 一時ディレクトリを削除
                fs::remove_dir_all(&self.temp_dir).ok();
            }
        }

        let _guard = EnvGuard {
            original: original_config_home,
            temp_dir: temp_dir.clone(),
        };

        // テスト用の設定ディレクトリを設定
        env::set_var("XDG_CONFIG_HOME", &temp_dir);

        // 環境変数が確実に設定されたことを確認
        assert_eq!(env::var("XDG_CONFIG_HOME").unwrap(), temp_dir.to_str().unwrap());

        // エイリアスマネージャーを作成してエイリアスを追加
        let mut manager = AliasManager::new();
        manager
            .add_alias(
                "test1".to_string(),
                PathBuf::from("/path/to/file1"),
                vec!["tag1".to_string()],
                Some("#FF0000".to_string()),
                true,
            )
            .unwrap();

        manager
            .add_alias(
                "test2".to_string(),
                PathBuf::from("/path/to/file2"),
                vec!["tag2".to_string(), "tag3".to_string()],
                None,
                false,
            )
            .unwrap();

        // 保存
        let save_result = manager.save();
        assert!(save_result.is_ok(), "保存に失敗しました: {:?}", save_result);

        // 保存されたファイルが存在することを確認
        let aliases_path = storage::get_aliases_path();
        assert!(aliases_path.is_ok(), "aliases_pathの取得に失敗: {:?}", aliases_path);
        let aliases_path = aliases_path.unwrap();
        assert!(aliases_path.exists(), "エイリアスファイルが存在しません: {:?}", aliases_path);

        // 新しいマネージャーで読み込み
        let mut new_manager = AliasManager::new();
        let load_result = new_manager.load();
        assert!(load_result.is_ok(), "読み込みに失敗しました: {:?}", load_result);

        // 読み込んだエイリアスの数を確認
        assert_eq!(new_manager.get_aliases().len(), 2, "読み込まれたエイリアス: {:?}", new_manager.get_aliases());

        // 1つ目のエイリアスを確認
        let alias1 = &new_manager.get_aliases()[0];
        assert_eq!(alias1.alias, "test1");
        assert_eq!(alias1.path, PathBuf::from("/path/to/file1"));
        assert_eq!(alias1.tags, vec!["tag1"]);
        assert_eq!(alias1.color, Some("#FF0000".to_string()));
        assert_eq!(alias1.is_favorite, true);

        // 2つ目のエイリアスを確認
        let alias2 = &new_manager.get_aliases()[1];
        assert_eq!(alias2.alias, "test2");
        assert_eq!(alias2.path, PathBuf::from("/path/to/file2"));
        assert_eq!(alias2.tags, vec!["tag2", "tag3"]);
        assert_eq!(alias2.color, None);
        assert_eq!(alias2.is_favorite, false);
    }

    #[test]
    fn test_load_empty_file() {
        use std::env;
        use std::fs;

        // 環境変数の競合を防ぐためにロックを取得
        let _lock = TEST_ENV_LOCK.lock().unwrap();

        // テスト用の一時ディレクトリを設定
        let temp_dir = env::temp_dir().join(format!("ofkt_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // 元の設定ディレクトリを保存
        let original_config_home = env::var("XDG_CONFIG_HOME").ok();

        // スコープガード: テスト終了時に必ず環境変数を復元
        struct EnvGuard {
            original: Option<String>,
            temp_dir: PathBuf,
        }

        impl Drop for EnvGuard {
            fn drop(&mut self) {
                // 元の環境変数に戻す
                if let Some(original) = &self.original {
                    env::set_var("XDG_CONFIG_HOME", original);
                } else {
                    env::remove_var("XDG_CONFIG_HOME");
                }
                // 一時ディレクトリを削除
                fs::remove_dir_all(&self.temp_dir).ok();
            }
        }

        let _guard = EnvGuard {
            original: original_config_home,
            temp_dir: temp_dir.clone(),
        };

        // テスト用の設定ディレクトリを設定
        env::set_var("XDG_CONFIG_HOME", &temp_dir);

        // エイリアスファイルが存在しない場合
        let mut manager = AliasManager::new();
        let load_result = manager.load();
        assert!(load_result.is_ok());
        assert_eq!(manager.get_aliases().len(), 0);
    }

    #[test]
    fn test_save_overwrites_previous_data() {
        use std::env;
        use std::fs;

        // 環境変数の競合を防ぐためにロックを取得
        let _lock = TEST_ENV_LOCK.lock().unwrap();

        // テスト用の一時ディレクトリを設定
        let temp_dir = env::temp_dir().join(format!("ofkt_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // 元の設定ディレクトリを保存
        let original_config_home = env::var("XDG_CONFIG_HOME").ok();

        // スコープガード: テスト終了時に必ず環境変数を復元
        struct EnvGuard {
            original: Option<String>,
            temp_dir: PathBuf,
        }

        impl Drop for EnvGuard {
            fn drop(&mut self) {
                // 元の環境変数に戻す
                if let Some(original) = &self.original {
                    env::set_var("XDG_CONFIG_HOME", original);
                } else {
                    env::remove_var("XDG_CONFIG_HOME");
                }
                // 一時ディレクトリを削除
                fs::remove_dir_all(&self.temp_dir).ok();
            }
        }

        let _guard = EnvGuard {
            original: original_config_home,
            temp_dir: temp_dir.clone(),
        };

        // テスト用の設定ディレクトリを設定
        env::set_var("XDG_CONFIG_HOME", &temp_dir);

        // 最初のデータを保存
        let mut manager1 = AliasManager::new();
        manager1
            .add_alias(
                "first".to_string(),
                PathBuf::from("/path/to/first"),
                vec![],
                None,
                false,
            )
            .unwrap();
        manager1.save().unwrap();

        // 新しいデータで上書き保存
        let mut manager2 = AliasManager::new();
        manager2
            .add_alias(
                "second".to_string(),
                PathBuf::from("/path/to/second"),
                vec![],
                None,
                false,
            )
            .unwrap();
        manager2.save().unwrap();

        // 読み込んで確認
        let mut manager3 = AliasManager::new();
        manager3.load().unwrap();

        // 最新のデータのみが存在することを確認
        assert_eq!(manager3.get_aliases().len(), 1);
        assert_eq!(manager3.get_aliases()[0].alias, "second");
    }

    #[test]
    fn test_toggle_favorite() {
        let mut manager = AliasManager::new();

        // エイリアスを追加（お気に入りでない）
        manager
            .add_alias(
                "test".to_string(),
                PathBuf::from("/path/to/file"),
                vec![],
                None,
                false,
            )
            .unwrap();

        let id = manager.get_aliases()[0].id.clone();

        // お気に入りに設定
        let result = manager.toggle_favorite(&id);
        assert!(result.is_ok());
        assert_eq!(manager.get_aliases()[0].is_favorite, true);

        // お気に入りを解除
        let result = manager.toggle_favorite(&id);
        assert!(result.is_ok());
        assert_eq!(manager.get_aliases()[0].is_favorite, false);

        // 再度お気に入りに設定
        let result = manager.toggle_favorite(&id);
        assert!(result.is_ok());
        assert_eq!(manager.get_aliases()[0].is_favorite, true);
    }

    #[test]
    fn test_toggle_favorite_nonexistent() {
        let mut manager = AliasManager::new();

        // エイリアスを追加
        manager
            .add_alias(
                "test".to_string(),
                PathBuf::from("/path/to/file"),
                vec![],
                None,
                false,
            )
            .unwrap();

        // 存在しないIDでお気に入りを切り替えようとする
        let result = manager.toggle_favorite("nonexistent-id");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "エイリアスID 'nonexistent-id' は存在しません"
        );
    }

    #[test]
    fn test_get_favorites_empty() {
        let manager = AliasManager::new();

        // お気に入りが空であること
        let favorites = manager.get_favorites();
        assert_eq!(favorites.len(), 0);
    }

    #[test]
    fn test_get_favorites_with_favorites() {
        let mut manager = AliasManager::new();

        // お気に入りのエイリアスを追加
        manager
            .add_alias(
                "favorite1".to_string(),
                PathBuf::from("/path/to/file1"),
                vec![],
                None,
                true,
            )
            .unwrap();

        manager
            .add_alias(
                "normal".to_string(),
                PathBuf::from("/path/to/file2"),
                vec![],
                None,
                false,
            )
            .unwrap();

        manager
            .add_alias(
                "favorite2".to_string(),
                PathBuf::from("/path/to/file3"),
                vec![],
                None,
                true,
            )
            .unwrap();

        // お気に入りのみが返されること
        let favorites = manager.get_favorites();
        assert_eq!(favorites.len(), 2);

        // お気に入りのエイリアス名を確認
        let favorite_aliases: Vec<&str> = favorites.iter().map(|a| a.alias.as_str()).collect();
        assert!(favorite_aliases.contains(&"favorite1"));
        assert!(favorite_aliases.contains(&"favorite2"));
        assert!(!favorite_aliases.contains(&"normal"));
    }

    #[test]
    fn test_get_favorites_after_toggle() {
        let mut manager = AliasManager::new();

        // エイリアスを追加（お気に入りでない）
        manager
            .add_alias(
                "test1".to_string(),
                PathBuf::from("/path/to/file1"),
                vec![],
                None,
                false,
            )
            .unwrap();

        manager
            .add_alias(
                "test2".to_string(),
                PathBuf::from("/path/to/file2"),
                vec![],
                None,
                false,
            )
            .unwrap();

        // 最初はお気に入りが空
        assert_eq!(manager.get_favorites().len(), 0);

        // test1をお気に入りに追加
        let id1 = manager.get_aliases()[0].id.clone();
        manager.toggle_favorite(&id1).unwrap();

        // お気に入りが1つ
        let favorites = manager.get_favorites();
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].alias, "test1");

        // test2もお気に入りに追加
        let id2 = manager.get_aliases()[1].id.clone();
        manager.toggle_favorite(&id2).unwrap();

        // お気に入りが2つ
        let favorites = manager.get_favorites();
        assert_eq!(favorites.len(), 2);

        // test1のお気に入りを解除
        manager.toggle_favorite(&id1).unwrap();

        // お気に入りが1つに戻る
        let favorites = manager.get_favorites();
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].alias, "test2");
    }
}
