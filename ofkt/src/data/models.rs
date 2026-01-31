use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// ファイルエイリアス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAlias {
    pub id: String,
    pub alias: String,
    pub path: PathBuf,
    #[serde(default)]
    pub tags: Vec<String>,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    #[serde(default)]
    pub is_favorite: bool,
}

/// ファイル履歴
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHistory {
    pub path: PathBuf,
    pub accessed_at: DateTime<Utc>,
    pub access_count: u32,
}

/// クイックアクセスエントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickAccessEntry {
    /// 一意なID
    pub id: String,
    /// 表示名
    pub name: String,
    /// パス
    pub path: PathBuf,
    /// 追加日時
    #[serde(with = "chrono::serde::ts_seconds")]
    pub added_at: DateTime<Utc>,
    /// 並び順（小さいほど上）
    pub order: u32,
    /// システム項目かどうか（ホーム、デスクトップなど）
    pub is_system: bool,
}

/// ファイルシステムのエントリ（ファイルまたはディレクトリ）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    /// エントリ名（ファイル名またはディレクトリ名）
    pub name: String,

    /// フルパス
    pub path: PathBuf,

    /// ディレクトリかどうか
    pub is_directory: bool,

    /// ファイルサイズ（バイト単位、ディレクトリの場合はNone）
    pub size: Option<u64>,

    /// 最終更新日時
    pub modified: Option<DateTime<Utc>>,

    /// 読み取り専用かどうか
    pub is_readonly: bool,

    /// 隠しファイル/フォルダかどうか
    pub is_hidden: bool,
}

impl DirectoryEntry {
    /// 基本的なコンストラクタ
    pub fn new(
        name: String,
        path: PathBuf,
        is_directory: bool,
        size: Option<u64>,
        modified: Option<DateTime<Utc>>,
        is_readonly: bool,
        is_hidden: bool,
    ) -> Self {
        Self {
            name,
            path,
            is_directory,
            size,
            modified,
            is_readonly,
            is_hidden,
        }
    }

    /// PathBufからDirectoryEntryを生成
    pub fn from_path(path: PathBuf) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(&path)?;
        let is_directory = metadata.is_dir();

        // ファイル名を取得（日本語などの非ASCII文字も正しく処理）
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // ファイルサイズ（ディレクトリの場合はNone）
        let size = if is_directory {
            None
        } else {
            Some(metadata.len())
        };

        // 最終更新日時
        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| {
                let duration = time.duration_since(std::time::UNIX_EPOCH).ok()?;
                DateTime::from_timestamp(duration.as_secs() as i64, 0)
            });

        // 読み取り専用かどうか
        let is_readonly = metadata.permissions().readonly();

        // 隠しファイル/フォルダかどうか（Windows環境での判定）
        #[cfg(target_os = "windows")]
        let is_hidden = {
            use std::os::windows::fs::MetadataExt;
            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
            (metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN) != 0
        };

        #[cfg(not(target_os = "windows"))]
        let is_hidden = name.starts_with('.');

        Ok(Self {
            name,
            path,
            is_directory,
            size,
            modified,
            is_readonly,
            is_hidden,
        })
    }

    /// ファイルかどうか
    pub fn is_file(&self) -> bool {
        !self.is_directory
    }

    /// WSLパスかどうかを判定
    pub fn is_wsl_path(&self) -> bool {
        self.path.to_string_lossy().starts_with(r"\\wsl")
    }
}

/// アプリケーション全体設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub window: WindowConfig,
    pub hotkey: HotkeyConfig,
    pub edge_trigger: EdgeTriggerConfig,
    pub autostart: AutostartConfig,
    pub theme: ThemeConfig,
    pub search: SearchConfig,
    pub file_operations: FileOperationConfig,
}

/// ウィンドウ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: f32,
    pub height: f32,
    pub position: WindowPosition,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default = "default_decorations")]
    pub decorations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPosition {
    pub x: f32,
    pub y: f32,
}

fn default_decorations() -> bool {
    true
}

/// ホットキー設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub enabled: bool,
    pub modifiers: Vec<String>,
    pub key: String,
}

/// 画面端トリガー設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeTriggerConfig {
    pub enabled: bool,
    pub edge: String,
    pub delay_ms: u64,
    pub trigger_width: i32,
}

/// 自動起動設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutostartConfig {
    pub enabled: bool,
}

/// テーマ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub mode: String,
    pub custom_accent_color: Option<String>,
}

/// 検索設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub incremental: bool,
    pub fuzzy_match: bool,
    pub search_paths: bool,
    pub search_aliases: bool,
    pub case_sensitive: bool,
}

/// ファイル操作設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperationConfig {
    pub confirm_delete: bool,
    pub use_trash: bool,
    pub default_open_action: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_file_alias_creation() {
        let now = Utc::now();
        let alias = FileAlias {
            id: "test-id".to_string(),
            alias: "test".to_string(),
            path: PathBuf::from("/path/to/file"),
            tags: vec!["tag1".to_string()],
            color: Some("#FF0000".to_string()),
            created_at: now,
            last_accessed: now,
            is_favorite: true,
        };

        assert_eq!(alias.id, "test-id");
        assert_eq!(alias.alias, "test");
        assert_eq!(alias.path, PathBuf::from("/path/to/file"));
        assert_eq!(alias.tags.len(), 1);
        assert_eq!(alias.color, Some("#FF0000".to_string()));
        assert_eq!(alias.is_favorite, true);
    }

    #[test]
    fn test_file_history_creation() {
        let now = Utc::now();
        let history = FileHistory {
            path: PathBuf::from("/path/to/file"),
            accessed_at: now,
            access_count: 5,
        };

        assert_eq!(history.path, PathBuf::from("/path/to/file"));
        assert_eq!(history.accessed_at, now);
        assert_eq!(history.access_count, 5);
    }

    #[test]
    fn test_file_history_serialization() {
        let now = Utc::now();
        let history = FileHistory {
            path: PathBuf::from("/path/to/file"),
            accessed_at: now,
            access_count: 3,
        };

        // JSON シリアライズ
        let json = serde_json::to_string(&history).unwrap();
        assert!(json.contains("path"));
        assert!(json.contains("accessed_at"));
        assert!(json.contains("access_count"));

        // JSON デシリアライズ
        let deserialized: FileHistory = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.path, history.path);
        assert_eq!(deserialized.access_count, history.access_count);
    }

    #[test]
    fn test_file_alias_serialization() {
        let now = Utc::now();
        let alias = FileAlias {
            id: "test-id".to_string(),
            alias: "test".to_string(),
            path: PathBuf::from("/path/to/file"),
            tags: vec![],
            color: None,
            created_at: now,
            last_accessed: now,
            is_favorite: false,
        };

        // JSON シリアライズ
        let json = serde_json::to_string(&alias).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("test"));

        // JSON デシリアライズ
        let deserialized: FileAlias = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, alias.id);
        assert_eq!(deserialized.alias, alias.alias);
        assert_eq!(deserialized.path, alias.path);
    }

    #[test]
    fn test_file_alias_with_empty_tags() {
        let now = Utc::now();
        let alias = FileAlias {
            id: "test-id".to_string(),
            alias: "test".to_string(),
            path: PathBuf::from("/path/to/file"),
            tags: vec![],
            color: None,
            created_at: now,
            last_accessed: now,
            is_favorite: false,
        };

        assert_eq!(alias.tags.len(), 0);
    }

    #[test]
    fn test_config_deserialization() {
        // デフォルト設定をロード
        let default_config = include_str!("../../config/default_config.json");
        let config: Config = serde_json::from_str(default_config).unwrap();

        assert!(!config.version.is_empty());
        assert!(config.window.width > 0.0);
        assert!(config.window.height > 0.0);
    }

    #[test]
    fn test_window_config() {
        let window_config = WindowConfig {
            width: 800.0,
            height: 600.0,
            position: WindowPosition { x: 100.0, y: 100.0 },
            always_on_top: true,
            decorations: false,
        };

        assert_eq!(window_config.width, 800.0);
        assert_eq!(window_config.height, 600.0);
        assert_eq!(window_config.position.x, 100.0);
        assert_eq!(window_config.position.y, 100.0);
        assert_eq!(window_config.always_on_top, true);
        assert_eq!(window_config.decorations, false);
    }

    #[test]
    fn test_hotkey_config() {
        let hotkey_config = HotkeyConfig {
            enabled: true,
            modifiers: vec!["Ctrl".to_string(), "Alt".to_string()],
            key: "Space".to_string(),
        };

        assert_eq!(hotkey_config.enabled, true);
        assert_eq!(hotkey_config.modifiers.len(), 2);
        assert_eq!(hotkey_config.key, "Space");
    }

    #[test]
    fn test_edge_trigger_config() {
        let edge_config = EdgeTriggerConfig {
            enabled: true,
            edge: "top".to_string(),
            delay_ms: 500,
            trigger_width: 10,
        };

        assert_eq!(edge_config.enabled, true);
        assert_eq!(edge_config.edge, "top");
        assert_eq!(edge_config.delay_ms, 500);
        assert_eq!(edge_config.trigger_width, 10);
    }

    #[test]
    fn test_autostart_config() {
        let autostart_config = AutostartConfig {
            enabled: false,
        };

        assert_eq!(autostart_config.enabled, false);
    }

    #[test]
    fn test_theme_config() {
        let theme_config = ThemeConfig {
            mode: "dark".to_string(),
            custom_accent_color: Some("#3B82F6".to_string()),
        };

        assert_eq!(theme_config.mode, "dark");
        assert_eq!(theme_config.custom_accent_color, Some("#3B82F6".to_string()));
    }

    #[test]
    fn test_search_config() {
        let search_config = SearchConfig {
            incremental: true,
            fuzzy_match: true,
            search_paths: true,
            search_aliases: true,
            case_sensitive: false,
        };

        assert_eq!(search_config.incremental, true);
        assert_eq!(search_config.fuzzy_match, true);
        assert_eq!(search_config.search_paths, true);
        assert_eq!(search_config.search_aliases, true);
        assert_eq!(search_config.case_sensitive, false);
    }

    #[test]
    fn test_file_operation_config() {
        let file_op_config = FileOperationConfig {
            confirm_delete: true,
            use_trash: true,
            default_open_action: "open".to_string(),
        };

        assert_eq!(file_op_config.confirm_delete, true);
        assert_eq!(file_op_config.use_trash, true);
        assert_eq!(file_op_config.default_open_action, "open");
    }

    #[test]
    fn test_config_serialization() {
        // デフォルト設定をロード
        let default_config = include_str!("../../config/default_config.json");
        let config: Config = serde_json::from_str(default_config).unwrap();

        // シリアライズ
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("version"));
        assert!(json.contains("window"));
        assert!(json.contains("hotkey"));

        // デシリアライズ
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, config.version);
    }

    #[test]
    fn test_default_decorations() {
        assert_eq!(default_decorations(), true);
    }

    #[test]
    fn test_directory_entry_creation() {
        let now = Utc::now();
        let entry = DirectoryEntry::new(
            "test_file.txt".to_string(),
            PathBuf::from("C:\\Users\\test\\test_file.txt"),
            false,
            Some(1024),
            Some(now),
            false,
            false,
        );

        assert_eq!(entry.name, "test_file.txt");
        assert_eq!(entry.path, PathBuf::from("C:\\Users\\test\\test_file.txt"));
        assert_eq!(entry.is_directory, false);
        assert_eq!(entry.size, Some(1024));
        assert_eq!(entry.modified, Some(now));
        assert_eq!(entry.is_readonly, false);
        assert_eq!(entry.is_hidden, false);
    }

    #[test]
    fn test_directory_entry_serialization() {
        let now = Utc::now();
        let entry = DirectoryEntry::new(
            "test_dir".to_string(),
            PathBuf::from("C:\\Users\\test\\test_dir"),
            true,
            None,
            Some(now),
            false,
            false,
        );

        // JSON シリアライズ
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("test_dir"));
        assert!(json.contains("is_directory"));

        // JSON デシリアライズ
        let deserialized: DirectoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, entry.name);
        assert_eq!(deserialized.path, entry.path);
        assert_eq!(deserialized.is_directory, entry.is_directory);
        assert_eq!(deserialized.size, entry.size);
    }

    #[test]
    fn test_directory_entry_is_file() {
        let file_entry = DirectoryEntry::new(
            "file.txt".to_string(),
            PathBuf::from("C:\\file.txt"),
            false,
            Some(512),
            None,
            false,
            false,
        );

        let dir_entry = DirectoryEntry::new(
            "dir".to_string(),
            PathBuf::from("C:\\dir"),
            true,
            None,
            None,
            false,
            false,
        );

        assert_eq!(file_entry.is_file(), true);
        assert_eq!(dir_entry.is_file(), false);
    }

    #[test]
    fn test_directory_entry_is_directory() {
        let dir_entry = DirectoryEntry::new(
            "test_dir".to_string(),
            PathBuf::from("C:\\test_dir"),
            true,
            None,
            None,
            false,
            false,
        );

        assert_eq!(dir_entry.is_directory, true);
        assert_eq!(dir_entry.is_file(), false);
    }

    #[test]
    fn test_directory_entry_from_path() {
        // テスト用の一時ファイルを作成
        let temp_dir = std::env::temp_dir();
        let test_file_path = temp_dir.join("ofkt_test_file.txt");

        // ファイルを作成
        std::fs::write(&test_file_path, "test content").unwrap();

        // from_pathでDirectoryEntryを生成
        let entry = DirectoryEntry::from_path(test_file_path.clone()).unwrap();

        assert_eq!(entry.name, "ofkt_test_file.txt");
        assert_eq!(entry.path, test_file_path);
        assert_eq!(entry.is_directory, false);
        assert!(entry.size.is_some());
        assert!(entry.size.unwrap() > 0);
        assert!(entry.modified.is_some());

        // クリーンアップ
        std::fs::remove_file(&test_file_path).ok();
    }

    #[test]
    fn test_directory_entry_from_path_directory() {
        // テスト用の一時ディレクトリを使用
        let temp_dir = std::env::temp_dir();

        // from_pathでDirectoryEntryを生成
        let entry = DirectoryEntry::from_path(temp_dir.clone()).unwrap();

        assert_eq!(entry.is_directory, true);
        assert_eq!(entry.size, None);
        assert!(entry.modified.is_some());
    }

    #[test]
    fn test_directory_entry_readonly() {
        let readonly_entry = DirectoryEntry::new(
            "readonly.txt".to_string(),
            PathBuf::from("C:\\readonly.txt"),
            false,
            Some(256),
            None,
            true,
            false,
        );

        assert_eq!(readonly_entry.is_readonly, true);
    }

    #[test]
    fn test_directory_entry_hidden() {
        let hidden_entry = DirectoryEntry::new(
            ".hidden_file".to_string(),
            PathBuf::from("C:\\.hidden_file"),
            false,
            Some(128),
            None,
            false,
            true,
        );

        assert_eq!(hidden_entry.is_hidden, true);
    }

    #[test]
    fn test_directory_entry_japanese_filename() {
        // 日本語ファイル名を含むテスト
        let temp_dir = std::env::temp_dir();
        let japanese_file_path = temp_dir.join("テストファイル.txt");

        // 日本語ファイル名でファイルを作成
        std::fs::write(&japanese_file_path, "日本語コンテンツ").unwrap();

        // from_pathでDirectoryEntryを生成
        let entry = DirectoryEntry::from_path(japanese_file_path.clone()).unwrap();

        // 日本語ファイル名が正しく取得できることを確認
        assert_eq!(entry.name, "テストファイル.txt");
        assert_eq!(entry.path, japanese_file_path);
        assert_eq!(entry.is_directory, false);
        assert!(entry.size.is_some());

        // クリーンアップ
        std::fs::remove_file(&japanese_file_path).ok();
    }

    #[test]
    fn test_directory_entry_japanese_directory() {
        // 日本語ディレクトリ名を含むテスト
        let temp_dir = std::env::temp_dir();
        let japanese_dir_path = temp_dir.join("テストディレクトリ");

        // 日本語ディレクトリ名でディレクトリを作成
        std::fs::create_dir(&japanese_dir_path).unwrap();

        // from_pathでDirectoryEntryを生成
        let entry = DirectoryEntry::from_path(japanese_dir_path.clone()).unwrap();

        // 日本語ディレクトリ名が正しく取得できることを確認
        assert_eq!(entry.name, "テストディレクトリ");
        assert_eq!(entry.path, japanese_dir_path);
        assert_eq!(entry.is_directory, true);
        assert_eq!(entry.size, None);

        // クリーンアップ
        std::fs::remove_dir(&japanese_dir_path).ok();
    }

    #[test]
    fn test_directory_entry_is_wsl_path() {
        use std::path::PathBuf;

        // WSLパスのテスト
        let wsl_entry = DirectoryEntry::new(
            "Ubuntu".to_string(),
            PathBuf::from(r"\\wsl$\Ubuntu"),
            true,
            None,
            None,
            false,
            false,
        );
        assert!(wsl_entry.is_wsl_path());

        let wsl_localhost_entry = DirectoryEntry::new(
            "home".to_string(),
            PathBuf::from(r"\\wsl.localhost\Ubuntu\home"),
            true,
            None,
            None,
            false,
            false,
        );
        assert!(wsl_localhost_entry.is_wsl_path());

        // 非WSLパスのテスト
        let normal_entry = DirectoryEntry::new(
            "Users".to_string(),
            PathBuf::from(r"C:\Users"),
            true,
            None,
            None,
            false,
            false,
        );
        assert!(!normal_entry.is_wsl_path());

        let network_entry = DirectoryEntry::new(
            "share".to_string(),
            PathBuf::from(r"\\network\share"),
            true,
            None,
            None,
            false,
            false,
        );
        assert!(!network_entry.is_wsl_path());
    }
}
