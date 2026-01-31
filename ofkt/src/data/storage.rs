use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use crate::data::models::{Config, FileAlias, FileHistory, QuickAccessEntry};

/// 設定ディレクトリのパスを取得
/// Linux: $HOME/.config/ofkt
/// Windows: %APPDATA%\Ofkt
pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("設定ディレクトリが見つかりません")?
        .join("ofkt");

    // ディレクトリが存在しない場合は作成
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .context("設定ディレクトリの作成に失敗しました")?;
    }

    Ok(config_dir)
}

/// 設定ファイルのパスを取得
pub fn get_config_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("config.json"))
}

/// エイリアスファイルのパスを取得
pub fn get_aliases_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("aliases.json"))
}

/// 履歴ファイルのパスを取得
pub fn get_history_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("history.json"))
}

/// クイックアクセスファイルのパスを取得
pub fn get_quick_access_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("quick_access.json"))
}

/// 設定ファイルを読み込む
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        // デフォルト設定ファイルから読み込む
        let default_config = include_str!("../../config/default_config.json");
        let config: Config = serde_json::from_str(default_config)
            .context("デフォルト設定の解析に失敗しました")?;

        // デフォルト設定を保存
        save_config(&config)?;

        return Ok(config);
    }

    let contents = fs::read_to_string(&config_path)
        .context("設定ファイルの読み込みに失敗しました")?;

    let config: Config = serde_json::from_str(&contents)
        .context("設定ファイルの解析に失敗しました")?;

    Ok(config)
}

/// 設定ファイルを保存（アトミック書き込み）
pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;
    let temp_path = config_path.with_extension("json.tmp");

    // 一時ファイルに書き込み
    let json = serde_json::to_string_pretty(config)
        .context("設定のシリアライズに失敗しました")?;

    fs::write(&temp_path, json)
        .context("一時ファイルの書き込みに失敗しました")?;

    // 一時ファイルをリネーム（アトミック操作）
    fs::rename(temp_path, config_path)
        .context("設定ファイルの保存に失敗しました")?;

    Ok(())
}

/// エイリアスファイルを読み込む
pub fn load_aliases() -> Result<Vec<FileAlias>> {
    let aliases_path = get_aliases_path()?;

    if !aliases_path.exists() {
        // エイリアスファイルが存在しない場合は初回起動としてサンプルデータを生成
        let sample_aliases = create_sample_aliases()?;

        // サンプルデータを保存
        save_aliases(&sample_aliases)?;

        return Ok(sample_aliases);
    }

    let contents = fs::read_to_string(&aliases_path)
        .context("エイリアスファイルの読み込みに失敗しました")?;

    let aliases: Vec<FileAlias> = serde_json::from_str(&contents)
        .context("エイリアスファイルの解析に失敗しました")?;

    Ok(aliases)
}

/// 初回起動時のサンプルエイリアスを生成
fn create_sample_aliases() -> Result<Vec<FileAlias>> {
    let now = chrono::Utc::now();
    let mut sample_aliases = Vec::new();

    // ドキュメントフォルダ
    if let Some(documents_dir) = dirs::document_dir() {
        sample_aliases.push(FileAlias {
            id: uuid::Uuid::new_v4().to_string(),
            alias: "ドキュメント".to_string(),
            path: documents_dir,
            tags: vec!["標準フォルダ".to_string()],
            color: Some("#3B82F6".to_string()), // 青色
            created_at: now,
            last_accessed: now,
            is_favorite: true,
        });
    }

    // ダウンロードフォルダ
    if let Some(downloads_dir) = dirs::download_dir() {
        sample_aliases.push(FileAlias {
            id: uuid::Uuid::new_v4().to_string(),
            alias: "ダウンロード".to_string(),
            path: downloads_dir,
            tags: vec!["標準フォルダ".to_string()],
            color: Some("#10B981".to_string()), // 緑色
            created_at: now,
            last_accessed: now,
            is_favorite: true,
        });
    }

    // デスクトップフォルダ
    if let Some(desktop_dir) = dirs::desktop_dir() {
        sample_aliases.push(FileAlias {
            id: uuid::Uuid::new_v4().to_string(),
            alias: "デスクトップ".to_string(),
            path: desktop_dir,
            tags: vec!["標準フォルダ".to_string()],
            color: Some("#F59E0B".to_string()), // オレンジ色
            created_at: now,
            last_accessed: now,
            is_favorite: true,
        });
    }

    Ok(sample_aliases)
}

/// エイリアスファイルを保存（アトミック書き込み）
pub fn save_aliases(aliases: &[FileAlias]) -> Result<()> {
    let aliases_path = get_aliases_path()?;
    let temp_path = aliases_path.with_extension("json.tmp");

    // 一時ファイルに書き込み
    let json = serde_json::to_string_pretty(aliases)
        .context("エイリアスのシリアライズに失敗しました")?;

    fs::write(&temp_path, json)
        .context("一時ファイルの書き込みに失敗しました")?;

    // 一時ファイルをリネーム（アトミック操作）
    fs::rename(temp_path, aliases_path)
        .context("エイリアスファイルの保存に失敗しました")?;

    Ok(())
}

/// 履歴ファイルを読み込む
pub fn load_history() -> Result<Vec<FileHistory>> {
    let history_path = get_history_path()?;

    if !history_path.exists() {
        // 履歴ファイルが存在しない場合は空のリストを返す
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(&history_path)
        .context("履歴ファイルの読み込みに失敗しました")?;

    let history: Vec<FileHistory> = serde_json::from_str(&contents)
        .context("履歴ファイルの解析に失敗しました")?;

    Ok(history)
}

/// 履歴ファイルを保存（アトミック書き込み）
pub fn save_history(history: &[FileHistory]) -> Result<()> {
    let history_path = get_history_path()?;
    let temp_path = history_path.with_extension("json.tmp");

    // 一時ファイルに書き込み
    let json = serde_json::to_string_pretty(history)
        .context("履歴のシリアライズに失敗しました")?;

    fs::write(&temp_path, json)
        .context("一時ファイルの書き込みに失敗しました")?;

    // 一時ファイルをリネーム（アトミック操作）
    fs::rename(temp_path, history_path)
        .context("履歴ファイルの保存に失敗しました")?;

    Ok(())
}

/// クイックアクセスを読み込む
pub fn load_quick_access() -> Result<Vec<QuickAccessEntry>> {
    let path = get_quick_access_path()?;

    if !path.exists() {
        // ファイルが存在しない場合はシステムデフォルトを生成
        return create_default_quick_access();
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("クイックアクセス読み込み失敗: {}", path.display()))?;

    let entries: Vec<QuickAccessEntry> = serde_json::from_str(&content)
        .with_context(|| format!("クイックアクセスのパースに失敗: {}", path.display()))?;

    Ok(entries)
}

/// クイックアクセスを保存（アトミック書き込み）
pub fn save_quick_access(entries: &[QuickAccessEntry]) -> Result<()> {
    let path = get_quick_access_path()?;
    let content = serde_json::to_string_pretty(entries)
        .context("クイックアクセスのシリアライズに失敗")?;

    // 親ディレクトリが存在しない場合は作成
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("ディレクトリ作成失敗: {}", parent.display()))?;
    }

    // アトミック書き込み
    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, &content)
        .with_context(|| format!("一時ファイル書き込み失敗: {}", temp_path.display()))?;
    std::fs::rename(&temp_path, &path)
        .with_context(|| format!("ファイルリネーム失敗: {} -> {}", temp_path.display(), path.display()))?;

    Ok(())
}

/// システムデフォルトのクイックアクセスを生成
fn create_default_quick_access() -> Result<Vec<QuickAccessEntry>> {
    use uuid::Uuid;

    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("ホームディレクトリの取得に失敗"))?;

    let mut entries = vec![];
    let now = chrono::Utc::now();

    // ホーム
    entries.push(QuickAccessEntry {
        id: Uuid::new_v4().to_string(),
        name: "ホーム".to_string(),
        path: home_dir.clone(),
        added_at: now,
        order: 0,
        is_system: true,
    });

    // デスクトップ
    if let Some(desktop) = dirs::desktop_dir() {
        entries.push(QuickAccessEntry {
            id: Uuid::new_v4().to_string(),
            name: "デスクトップ".to_string(),
            path: desktop,
            added_at: now,
            order: 1,
            is_system: true,
        });
    }

    // ドキュメント
    if let Some(documents) = dirs::document_dir() {
        entries.push(QuickAccessEntry {
            id: Uuid::new_v4().to_string(),
            name: "ドキュメント".to_string(),
            path: documents,
            added_at: now,
            order: 2,
            is_system: true,
        });
    }

    // ダウンロード
    if let Some(downloads) = dirs::download_dir() {
        entries.push(QuickAccessEntry {
            id: Uuid::new_v4().to_string(),
            name: "ダウンロード".to_string(),
            path: downloads,
            added_at: now,
            order: 3,
            is_system: true,
        });
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::sync::Mutex;

    // テスト間で環境変数の設定が競合しないように、テストを直列化するためのロック
    static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_get_config_dir() {
        let config_dir = get_config_dir();
        assert!(config_dir.is_ok());
    }

    #[test]
    fn test_get_config_dir_creates_directory() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();

        // テスト用の一時ディレクトリを設定
        let temp_dir = env::temp_dir().join(format!("ofkt_storage_test_{}", uuid::Uuid::new_v4()));
        let original_config_home = env::var("XDG_CONFIG_HOME").ok();

        // スコープガード
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

        let config_dir = get_config_dir().unwrap();
        assert!(config_dir.exists());
        assert!(config_dir.ends_with("ofkt"));
    }

    #[test]
    fn test_get_config_path() {
        let config_path = get_config_path();
        assert!(config_path.is_ok());
        assert!(config_path.unwrap().ends_with("config.json"));
    }

    #[test]
    fn test_get_aliases_path() {
        let aliases_path = get_aliases_path();
        assert!(aliases_path.is_ok());
        assert!(aliases_path.unwrap().ends_with("aliases.json"));
    }

    #[test]
    fn test_get_history_path() {
        let history_path = get_history_path();
        assert!(history_path.is_ok());
        assert!(history_path.unwrap().ends_with("history.json"));
    }

    #[test]
    fn test_load_config_with_default() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();

        let temp_dir = env::temp_dir().join(format!("ofkt_config_test_{}", uuid::Uuid::new_v4()));
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

        // 設定ファイルが存在しない場合、デフォルト設定がロードされるはず
        let config = load_config().unwrap();
        assert!(!config.version.is_empty());

        // 設定ファイルが作成されていることを確認
        let config_path = get_config_path().unwrap();
        assert!(config_path.exists());
    }

    #[test]
    fn test_save_and_load_config() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();

        let temp_dir = env::temp_dir().join(format!("ofkt_save_config_test_{}", uuid::Uuid::new_v4()));
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

        // デフォルト設定をロード
        let mut config = load_config().unwrap();

        // 設定を変更
        config.window.width = 1024.0;
        config.window.height = 768.0;

        // 保存
        let save_result = save_config(&config);
        assert!(save_result.is_ok());

        // 再度ロード
        let loaded_config = load_config().unwrap();
        assert_eq!(loaded_config.window.width, 1024.0);
        assert_eq!(loaded_config.window.height, 768.0);
    }

    #[test]
    fn test_load_aliases_creates_sample_data() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();

        let temp_dir = env::temp_dir().join(format!("ofkt_aliases_test_{}", uuid::Uuid::new_v4()));
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

        // エイリアスファイルが存在しない場合、サンプルデータが生成されるはず
        let aliases = load_aliases().unwrap();

        // サンプルデータは最大3つ（ドキュメント、ダウンロード、デスクトップ）
        // 環境によっては全てのディレクトリが取得できない可能性があるため、
        // 少なくとも1つは生成されることを確認
        assert!(aliases.len() > 0);
        assert!(aliases.len() <= 3);

        // 生成されたエイリアスの基本的なプロパティを検証
        for alias in &aliases {
            assert!(!alias.id.is_empty());
            assert!(!alias.alias.is_empty());
            assert!(alias.path.exists() || cfg!(windows)); // Windowsでは存在確認をスキップ
            assert!(alias.tags.contains(&"標準フォルダ".to_string()));
            assert!(alias.color.is_some());
            assert_eq!(alias.is_favorite, true);
        }

        // エイリアスファイルが作成されていることを確認
        let aliases_path = get_aliases_path().unwrap();
        assert!(aliases_path.exists());

        // 再度読み込んで同じデータが取得できることを確認
        let reloaded_aliases = load_aliases().unwrap();
        assert_eq!(reloaded_aliases.len(), aliases.len());
    }

    #[test]
    fn test_save_and_load_aliases() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();

        let temp_dir = env::temp_dir().join(format!("ofkt_save_aliases_test_{}", uuid::Uuid::new_v4()));
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

        // テストデータを作成
        let now = chrono::Utc::now();
        let test_aliases = vec![
            FileAlias {
                id: uuid::Uuid::new_v4().to_string(),
                alias: "test1".to_string(),
                path: PathBuf::from("/path/to/test1"),
                tags: vec!["tag1".to_string()],
                color: Some("#FF0000".to_string()),
                created_at: now,
                last_accessed: now,
                is_favorite: true,
            },
            FileAlias {
                id: uuid::Uuid::new_v4().to_string(),
                alias: "test2".to_string(),
                path: PathBuf::from("/path/to/test2"),
                tags: vec![],
                color: None,
                created_at: now,
                last_accessed: now,
                is_favorite: false,
            },
        ];

        // 保存
        let save_result = save_aliases(&test_aliases);
        assert!(save_result.is_ok());

        // ロード
        let loaded_aliases = load_aliases().unwrap();
        assert_eq!(loaded_aliases.len(), 2);
        assert_eq!(loaded_aliases[0].alias, "test1");
        assert_eq!(loaded_aliases[1].alias, "test2");
    }

    #[test]
    fn test_atomic_save_config() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();

        let temp_dir = env::temp_dir().join(format!("ofkt_atomic_test_{}", uuid::Uuid::new_v4()));
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

        let config = load_config().unwrap();
        save_config(&config).unwrap();

        // 一時ファイルが削除されていることを確認
        let temp_path = get_config_path().unwrap().with_extension("json.tmp");
        assert!(!temp_path.exists());
    }

    #[test]
    fn test_atomic_save_aliases() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();

        let temp_dir = env::temp_dir().join(format!("ofkt_atomic_alias_test_{}", uuid::Uuid::new_v4()));
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

        let aliases = vec![];
        save_aliases(&aliases).unwrap();

        // 一時ファイルが削除されていることを確認
        let temp_path = get_aliases_path().unwrap().with_extension("json.tmp");
        assert!(!temp_path.exists());
    }

    #[test]
    fn test_load_history_empty() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();

        let temp_dir = env::temp_dir().join(format!("ofkt_history_test_{}", uuid::Uuid::new_v4()));
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

        // 履歴ファイルが存在しない場合、空のベクターが返されるはず
        let history = load_history().unwrap();
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_save_and_load_history() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();

        let temp_dir = env::temp_dir().join(format!("ofkt_save_history_test_{}", uuid::Uuid::new_v4()));
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

        // テストデータを作成
        let now = chrono::Utc::now();
        use crate::data::models::FileHistory;
        let test_history = vec![
            FileHistory {
                path: PathBuf::from("/path/to/file1"),
                accessed_at: now,
                access_count: 5,
            },
            FileHistory {
                path: PathBuf::from("/path/to/file2"),
                accessed_at: now,
                access_count: 3,
            },
        ];

        // 保存
        let save_result = save_history(&test_history);
        assert!(save_result.is_ok());

        // ロード
        let loaded_history = load_history().unwrap();
        assert_eq!(loaded_history.len(), 2);
        assert_eq!(loaded_history[0].path, PathBuf::from("/path/to/file1"));
        assert_eq!(loaded_history[0].access_count, 5);
        assert_eq!(loaded_history[1].path, PathBuf::from("/path/to/file2"));
        assert_eq!(loaded_history[1].access_count, 3);
    }

    #[test]
    fn test_atomic_save_history() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();

        let temp_dir = env::temp_dir().join(format!("ofkt_atomic_history_test_{}", uuid::Uuid::new_v4()));
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

        let history = vec![];
        save_history(&history).unwrap();

        // 一時ファイルが削除されていることを確認
        let temp_path = get_history_path().unwrap().with_extension("json.tmp");
        assert!(!temp_path.exists());
    }

    #[test]
    fn test_create_sample_aliases() {
        // サンプルエイリアスの生成をテスト
        let sample_aliases = create_sample_aliases().unwrap();

        // 少なくとも1つは生成されるはず
        assert!(sample_aliases.len() > 0);
        assert!(sample_aliases.len() <= 3);

        // 各エイリアスの基本的なプロパティを検証
        for alias in &sample_aliases {
            // IDが有効なUUIDであることを確認
            assert!(uuid::Uuid::parse_str(&alias.id).is_ok());

            // エイリアス名が空でないことを確認
            assert!(!alias.alias.is_empty());

            // パスが空でないことを確認
            assert!(alias.path.to_str().is_some());

            // タグに「標準フォルダ」が含まれることを確認
            assert!(alias.tags.contains(&"標準フォルダ".to_string()));

            // 色が設定されていることを確認
            assert!(alias.color.is_some());
            let color = alias.color.as_ref().unwrap();
            assert!(color.starts_with('#'));
            assert_eq!(color.len(), 7);

            // お気に入りフラグがtrueであることを確認
            assert_eq!(alias.is_favorite, true);

            // 作成日時とアクセス日時が設定されていることを確認
            // （未来の日時でないことを確認）
            let now = chrono::Utc::now();
            assert!(alias.created_at <= now);
            assert!(alias.last_accessed <= now);
        }

        // 生成されたエイリアス名を確認
        let alias_names: Vec<&str> = sample_aliases.iter().map(|a| a.alias.as_str()).collect();
        assert!(alias_names.contains(&"ドキュメント") ||
                alias_names.contains(&"ダウンロード") ||
                alias_names.contains(&"デスクトップ"));
    }
}
