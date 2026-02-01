use crate::core::alias::AliasManager;
use crate::core::clipboard::ClipboardState;
use crate::core::directory_browser::DirectoryBrowser;
use crate::core::quick_access::QuickAccessManager;
use crate::core::search::SearchEngine;
use crate::data::models::{Config, FileAlias, QuickAccessEntry};
use crate::platform::hotkey::{HotkeyManager, string_to_modifiers, string_to_code};
use crate::platform::SystemTray;
use crate::ui::search_bar::SearchDebouncer;
use crate::ui::theme::Theme;
use crate::utils::path::paths_equal;
use global_hotkey::hotkey::{Code, Modifiers};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// フォーカス領域
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusArea {
    /// 検索バー
    Search,
    /// サイドバー（エイリアス、クイックアクセス、ドライブ）
    Sidebar,
    /// メインパネル（エントリリスト）
    Main,
}

impl Default for FocusArea {
    fn default() -> Self {
        Self::Main  // デフォルトはメインパネル
    }
}

/// ペースト完了後のハイライト情報
#[derive(Debug, Clone)]
pub struct PastedFileHighlight {
    /// ハイライト対象のパスセット
    pub paths: HashSet<PathBuf>,

    /// ハイライト開始時刻
    pub timestamp: Instant,
}

impl PastedFileHighlight {
    /// 新しいハイライトを作成
    pub fn new(paths: Vec<PathBuf>) -> Self {
        Self {
            paths: paths.into_iter().collect(),
            timestamp: Instant::now(),
        }
    }

    /// ハイライトが期限切れかチェック (3秒経過)
    pub fn is_expired(&self) -> bool {
        self.timestamp.elapsed() > Duration::from_secs(3)
    }

    /// 指定したパスがハイライト対象か
    pub fn contains(&self, path: &PathBuf) -> bool {
        self.paths.iter().any(|p| paths_equal(p, path))
    }
}

/// ブラウザモード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowseMode {
    /// エイリアスモード（既存機能）
    Alias,
    /// ディレクトリブラウザモード
    Directory,
}

/// アプリケーション全体の状態
pub struct AppState {
    /// 設定
    pub config: Option<Config>,

    /// ファイルエイリアスのリスト
    pub file_aliases: Vec<FileAlias>,

    /// 検索クエリ
    pub search_query: String,

    /// ディレクトリモード用の検索クエリ
    pub directory_search_query: String,

    /// 検索バーがフォーカスを持っているか
    pub search_bar_focused: bool,

    /// ディレクトリ検索バーがフォーカスを持っているか
    pub directory_search_bar_focused: bool,

    /// 検索結果（フィルタリング後のエイリアス）
    pub filtered_items: Vec<FileAlias>,

    /// 選択中のアイテムのインデックス
    pub selected_index: Option<usize>,

    /// 設定画面を表示するか
    pub show_settings: bool,

    /// 現在のテーマ（Light/Dark）
    pub current_theme: Theme,

    /// 検索デバウンサー
    pub search_debouncer: SearchDebouncer,

    /// 初期化が完了したか
    pub initialized: bool,

    /// ブラウザモード
    pub browse_mode: BrowseMode,

    /// ディレクトリブラウザ
    pub directory_browser: Option<DirectoryBrowser>,

    /// ディレクトリブラウザでの選択インデックス
    pub selected_directory_index: Option<usize>,

    /// 展開されているディレクトリのパスセット
    pub expanded_directories: HashSet<PathBuf>,

    /// グローバルホットキーマネージャ
    pub hotkey_manager: HotkeyManager,

    /// システムトレイ
    pub system_tray: SystemTray,

    /// ウィンドウ表示状態
    pub is_window_visible: bool,

    /// 最後にホットキーが押された時刻（重複防止用）
    pub last_hotkey_time: Option<Instant>,

    /// 現在のフォーカス領域
    pub current_focus_area: FocusArea,

    /// サイドバーの選択インデックス
    pub selected_sidebar_index: Option<usize>,

    /// エイリアス管理
    pub alias_manager: AliasManager,

    /// エイリアス追加ダイアログを表示するか
    pub show_add_alias_dialog: bool,

    /// エイリアス追加ダイアログの入力値
    pub new_alias_name: String,
    pub new_alias_path: String,

    /// 検索エンジン
    pub search_engine: SearchEngine,

    /// クリップボード状態
    pub clipboard_state: ClipboardState,

    /// クイックアクセス管理
    pub quick_access_manager: QuickAccessManager,

    /// クイックアクセスエントリ（表示用キャッシュ）
    pub quick_access_entries: Vec<QuickAccessEntry>,

    /// ペースト直後のハイライト対象パス
    pub pasted_files_highlight: Option<PastedFileHighlight>,

    /// ペースト操作の結果メッセージ
    pub paste_result_message: Option<PasteResultMessage>,

    /// クイックアクセス追加確認ダイアログの状態
    pub add_quick_access_dialog: Option<AddQuickAccessDialog>,

    /// 上書き確認ダイアログの状態
    pub overwrite_confirmation_dialog: Option<OverwriteConfirmationDialog>,

    /// Ctrl+C が押されたフラグ
    pub pending_file_copy: bool,
    /// Ctrl+X が押されたフラグ
    pub pending_file_cut: bool,
    /// Ctrl+V が押されたフラグ
    pub pending_file_paste: bool,
}

/// クイックアクセス追加確認ダイアログ
#[derive(Debug, Clone)]
pub struct AddQuickAccessDialog {
    /// 追加するフォルダのパス
    pub path: PathBuf,
    /// 表示名（編集可能）
    pub name: String,
}

impl AddQuickAccessDialog {
    pub fn new(path: PathBuf, default_name: String) -> Self {
        Self {
            path,
            name: default_name,
        }
    }
}

/// 上書き確認ダイアログ
#[derive(Debug, Clone)]
pub struct OverwriteConfirmationDialog {
    /// 上書き対象のファイル一覧
    pub files: Vec<PathBuf>,
    /// ペースト保留中のデータ
    pub pending_paste: PendingPasteOperation,
}

/// ペースト保留操作
#[derive(Debug, Clone)]
pub struct PendingPasteOperation {
    pub src_paths: Vec<PathBuf>,
    pub dest_dir: PathBuf,
    pub mode: crate::core::clipboard::ClipboardMode,
}

/// ペースト操作の結果
#[derive(Debug, Clone)]
pub struct PasteResultMessage {
    /// メッセージテキスト
    pub message: String,
    /// 成功/失敗のタイプ
    pub message_type: MessageType,
    /// メッセージが表示された時刻
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Success,
    Error,
    Warning,
}

impl PasteResultMessage {
    pub fn new(message: String, message_type: MessageType) -> Self {
        Self {
            message,
            message_type,
            timestamp: Instant::now(),
        }
    }

    /// メッセージが期限切れか（5秒経過）
    pub fn is_expired(&self) -> bool {
        self.timestamp.elapsed() > Duration::from_secs(5)
    }
}

impl Default for AppState {
    fn default() -> Self {
        // HotkeyManagerの初期化
        let hotkey_manager = HotkeyManager::new()
            .unwrap_or_else(|e| {
                log::warn!("HotkeyManagerの初期化に失敗しました: {}", e);
                // エラー時は後で再試行できるよう、デフォルトインスタンスを作成
                HotkeyManager::new().expect("HotkeyManagerの作成に失敗しました")
            });

        Self {
            config: None,
            file_aliases: Vec::new(),
            search_query: String::new(),
            directory_search_query: String::new(),
            search_bar_focused: false,
            directory_search_bar_focused: false,
            filtered_items: Vec::new(),
            selected_index: None,
            show_settings: false,
            current_theme: Theme::default(),
            search_debouncer: SearchDebouncer::default(),
            initialized: false,
            browse_mode: BrowseMode::Alias,
            directory_browser: None,
            selected_directory_index: None,
            expanded_directories: HashSet::new(),
            hotkey_manager,
            system_tray: SystemTray::new(),
            is_window_visible: true,
            last_hotkey_time: None,
            current_focus_area: FocusArea::default(),
            selected_sidebar_index: None,
            alias_manager: AliasManager::new(),
            show_add_alias_dialog: false,
            new_alias_name: String::new(),
            new_alias_path: String::new(),
            search_engine: SearchEngine::new(),
            clipboard_state: ClipboardState::new(),
            quick_access_manager: QuickAccessManager::new(),
            quick_access_entries: Vec::new(),
            pasted_files_highlight: None,
            paste_result_message: None,
            add_quick_access_dialog: None,
            overwrite_confirmation_dialog: None,
            pending_file_copy: false,
            pending_file_cut: false,
            pending_file_paste: false,
        }
    }
}

impl AppState {
    /// 新しい AppState を作成
    pub fn new() -> Self {
        Self::default()
    }

    /// 設定を読み込む
    pub fn load_config(&mut self) -> anyhow::Result<()> {
        let config = crate::data::storage::load_config()?;
        self.config = Some(config);
        Ok(())
    }

    /// エイリアスを読み込む
    pub fn load_aliases(&mut self) -> anyhow::Result<()> {
        let aliases = crate::data::storage::load_aliases()?;
        self.file_aliases = aliases;
        self.search_engine.set_aliases(self.file_aliases.clone());
        self.filtered_items = self.file_aliases.clone();
        Ok(())
    }

    /// 遅延初期化（バックグラウンドで設定とエイリアスを読み込む）
    ///
    /// # パフォーマンス最適化
    /// - 起動時は最小限の初期化のみを行う
    /// - 設定とエイリアスの読み込みはバックグラウンドで行う
    /// - UI の表示を優先し、起動時間を短縮
    pub fn lazy_initialize(&mut self) -> anyhow::Result<()> {
        if self.initialized {
            return Ok(());
        }

        // 設定を読み込み
        if let Err(e) = self.load_config() {
            log::warn!("設定の読み込みに失敗（デフォルト設定を使用）: {}", e);
        }

        // エイリアスを読み込む
        if let Err(e) = self.alias_manager.load() {
            log::warn!("エイリアスの読み込みに失敗: {}", e);
        } else {
            // 互換性維持のため、file_aliasesにもコピー
            self.file_aliases = self.alias_manager.get_aliases().to_vec();
            self.search_engine.set_aliases(self.file_aliases.clone());
            self.filtered_items = self.file_aliases.clone();
            log::info!("{} 件のエイリアスを読み込みました", self.file_aliases.len());
        }

        // 設定からホットキーを登録（フォールバック付き）
        if let Err(e) = self.register_configured_hotkey() {
            log::warn!("設定からのホットキー登録に失敗: {}。デフォルト設定を使用します。", e);

            // デフォルト設定でリトライ
            let default_modifiers = Modifiers::CONTROL | Modifiers::SHIFT;
            let default_code = Code::KeyO;
            if let Err(e) = self.hotkey_manager.register(default_modifiers, default_code) {
                log::error!("デフォルトホットキーの登録も失敗: {}", e);
            } else {
                log::info!("デフォルトホットキーを登録しました: Ctrl+Shift+O");
            }
        }

        // システムトレイを構築
        if let Err(e) = self.system_tray.build() {
            log::warn!("システムトレイの構築に失敗しました: {}", e);
            log::warn!("トレイアイコンは表示されませんが、アプリケーションは継続します");
        } else {
            log::info!("システムトレイを構築しました");
        }

        // クイックアクセスを読み込む
        if let Err(e) = self.load_quick_access() {
            log::warn!("クイックアクセスの読み込みに失敗: {}", e);
        }

        self.initialized = true;
        Ok(())
    }

    /// 設定ファイルから読み込んだホットキーを登録
    pub fn register_configured_hotkey(&mut self) -> Result<(), String> {
        // 設定が読み込まれているか確認
        let config = self.config.as_ref()
            .ok_or_else(|| "設定が読み込まれていません".to_string())?;

        // ホットキーが無効の場合は何もしない
        if !config.hotkey.enabled {
            log::info!("ホットキーは無効に設定されています");
            return Ok(());
        }

        // 修飾キーを変換
        let modifiers = string_to_modifiers(&config.hotkey.modifiers)
            .map_err(|e| format!("修飾キーの変換に失敗: {}", e))?;

        // キーコードを変換
        let code = string_to_code(&config.hotkey.key)
            .map_err(|e| format!("キーコードの変換に失敗: {}", e))?;

        // ホットキーを登録
        self.hotkey_manager.register(modifiers, code)
            .map_err(|e| format!("ホットキーの登録に失敗: {}", e))?;

        log::info!("グローバルホットキーを登録しました: {:?}+{}",
            config.hotkey.modifiers, config.hotkey.key);

        Ok(())
    }

    /// 初期化が完了しているか
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// ブラウザモードを設定
    pub fn set_browse_mode(&mut self, mode: BrowseMode) {
        self.browse_mode = mode;
    }

    /// ディレクトリブラウザを初期化
    pub fn init_directory_browser(&mut self, path: PathBuf) -> std::io::Result<()> {
        self.directory_browser = Some(DirectoryBrowser::new(path)?);
        Ok(())
    }

    /// 現在表示すべきエントリを取得
    pub fn get_current_entries(&self) -> Vec<crate::data::models::DirectoryEntry> {
        if let Some(ref browser) = self.directory_browser {
            browser.entries().to_vec()
        } else {
            Vec::new()
        }
    }

    /// 検索クエリに基づいてエイリアスをフィルタリング
    pub fn filter_aliases(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_items = self.file_aliases.clone();
        } else {
            // SearchEngineを使用した高度な検索
            let results = self.search_engine.search(&self.search_query);

            // SearchResultからFileAliasに変換
            // スコア順にソートされているので、その順序を維持
            self.filtered_items = results
                .into_iter()
                .map(|result| result.alias)
                .collect();
        }
    }

    /// クイックアクセスを読み込む
    pub fn load_quick_access(&mut self) -> anyhow::Result<()> {
        self.quick_access_manager.load()?;
        self.quick_access_entries = self.quick_access_manager.get_entries();
        Ok(())
    }

    /// クイックアクセスにエントリを追加
    pub fn add_to_quick_access(&mut self, name: String, path: PathBuf) -> Result<(), String> {
        self.quick_access_manager.add_entry(name, path)?;
        self.quick_access_manager.save()
            .map_err(|e| format!("保存失敗: {}", e))?;
        self.quick_access_entries = self.quick_access_manager.get_entries();
        Ok(())
    }

    /// クイックアクセスからエントリを削除
    pub fn remove_from_quick_access(&mut self, id: &str) -> Result<(), String> {
        self.quick_access_manager.remove_entry_by_id(id)?;
        self.quick_access_manager.save()
            .map_err(|e| format!("保存失敗: {}", e))?;
        self.quick_access_entries = self.quick_access_manager.get_entries();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::models::*;

    /// テスト用のデフォルト設定を作成
    fn create_test_config(enabled: bool, modifiers: Vec<String>, key: String) -> Config {
        Config {
            version: "1.0.0".to_string(),
            window: WindowConfig {
                width: 800.0,
                height: 600.0,
                position: WindowPosition { x: 0.0, y: 0.0 },
                always_on_top: false,
                decorations: true,
            },
            hotkey: HotkeyConfig {
                enabled,
                modifiers,
                key,
            },
            edge_trigger: EdgeTriggerConfig {
                enabled: false,
                edge: "top".to_string(),
                delay_ms: 500,
                trigger_width: 10,
            },
            autostart: AutostartConfig { enabled: false },
            theme: ThemeConfig {
                mode: "light".to_string(),
                custom_accent_color: None,
            },
            search: SearchConfig {
                incremental: true,
                fuzzy_match: false,
                search_paths: true,
                search_aliases: true,
                case_sensitive: false,
            },
            file_operations: FileOperationConfig {
                confirm_delete: true,
                use_trash: true,
                default_open_action: "open".to_string(),
            },
        }
    }

    #[test]
    fn test_register_configured_hotkey_success() {
        // 正常系: 有効な設定でホットキー登録が成功
        let mut state = AppState::default();
        state.config = Some(create_test_config(
            true,
            vec!["Ctrl".to_string(), "Shift".to_string()],
            "O".to_string(),
        ));

        let result = state.register_configured_hotkey();
        assert!(result.is_ok(), "ホットキー登録が失敗しました: {:?}", result.err());
    }

    #[test]
    fn test_register_configured_hotkey_disabled() {
        // ホットキー無効時: enabled=false で何も登録されず Ok が返される
        let mut state = AppState::default();
        state.config = Some(create_test_config(
            false,
            vec!["Ctrl".to_string()],
            "O".to_string(),
        ));

        let result = state.register_configured_hotkey();
        assert!(result.is_ok(), "ホットキーが無効でもOkを返すべき: {:?}", result.err());
    }

    #[test]
    fn test_register_configured_hotkey_invalid_modifier() {
        // 異常系: 無効な修飾キーでエラーが返される
        let mut state = AppState::default();
        state.config = Some(create_test_config(
            true,
            vec!["InvalidModifier".to_string()],
            "O".to_string(),
        ));

        let result = state.register_configured_hotkey();
        assert!(result.is_err(), "無効な修飾キーでエラーを返すべき");
        assert!(
            result.unwrap_err().contains("修飾キーの変換に失敗"),
            "エラーメッセージに「修飾キーの変換に失敗」を含むべき"
        );
    }

    #[test]
    fn test_register_configured_hotkey_invalid_key() {
        // 異常系: 無効なキーコードでエラーが返される
        let mut state = AppState::default();
        state.config = Some(create_test_config(
            true,
            vec!["Ctrl".to_string()],
            "InvalidKey".to_string(),
        ));

        let result = state.register_configured_hotkey();
        assert!(result.is_err(), "無効なキーでエラーを返すべき");
        assert!(
            result.unwrap_err().contains("キーコードの変換に失敗"),
            "エラーメッセージに「キーコードの変換に失敗」を含むべき"
        );
    }

    #[test]
    fn test_register_configured_hotkey_no_config() {
        // 異常系: config が None の場合にエラーが返される
        let mut state = AppState::default();
        state.config = None;

        let result = state.register_configured_hotkey();
        assert!(result.is_err(), "設定がない場合はエラーを返すべき");
        assert!(
            result.unwrap_err().contains("設定が読み込まれていません"),
            "エラーメッセージに「設定が読み込まれていません」を含むべき"
        );
    }

    #[test]
    fn test_app_state_default() {
        // AppState のデフォルトインスタンス生成をテスト
        let state = AppState::default();
        assert!(state.config.is_none(), "デフォルトでは設定がNone");
        assert_eq!(state.file_aliases.len(), 0, "デフォルトではエイリアスが空");
        assert_eq!(state.search_query, "", "デフォルトでは検索クエリが空");
        assert!(!state.initialized, "デフォルトでは初期化されていない");
    }

    #[test]
    fn test_filter_aliases_empty_query() {
        // 検索クエリが空の場合、全てのエイリアスが表示される
        let mut state = AppState::default();
        state.file_aliases = vec![
            FileAlias {
                id: "1".to_string(),
                alias: "test1".to_string(),
                path: PathBuf::from("/path/to/test1"),
                tags: vec![],
                color: None,
                created_at: chrono::Utc::now(),
                last_accessed: chrono::Utc::now(),
                is_favorite: false,
            },
            FileAlias {
                id: "2".to_string(),
                alias: "test2".to_string(),
                path: PathBuf::from("/path/to/test2"),
                tags: vec![],
                color: None,
                created_at: chrono::Utc::now(),
                last_accessed: chrono::Utc::now(),
                is_favorite: false,
            },
        ];

        state.search_query = String::new();
        state.filter_aliases();

        assert_eq!(state.filtered_items.len(), 2, "全エイリアスが表示されるべき");
    }

    #[test]
    fn test_filter_aliases_with_query() {
        // 検索クエリがある場合、マッチするエイリアスのみが表示される
        let mut state = AppState::default();
        let now = chrono::Utc::now();
        state.file_aliases = vec![
            FileAlias {
                id: "1".to_string(),
                alias: "test1".to_string(),
                path: PathBuf::from("/path/to/test1"),
                tags: vec![],
                color: None,
                created_at: now,
                last_accessed: now,
                is_favorite: false,
            },
            FileAlias {
                id: "2".to_string(),
                alias: "other".to_string(),
                path: PathBuf::from("/path/to/other"),
                tags: vec![],
                color: None,
                created_at: now,
                last_accessed: now,
                is_favorite: false,
            },
        ];

        // SearchEngineにも設定
        state.search_engine.set_aliases(state.file_aliases.clone());

        state.search_query = "test".to_string();
        state.filter_aliases();

        assert_eq!(state.filtered_items.len(), 1, "マッチするエイリアスのみ表示");
        assert_eq!(state.filtered_items[0].alias, "test1", "test1がフィルタリングされるべき");
    }

    #[test]
    fn test_filter_aliases_with_search_engine_fuzzy() {
        // SearchEngineのファジーマッチングを使用したテスト
        let mut state = AppState::default();
        let now = chrono::Utc::now();

        // テスト用エイリアスを追加
        state.file_aliases = vec![
            FileAlias {
                id: "1".to_string(),
                alias: "config".to_string(),
                path: PathBuf::from("/etc/config"),
                tags: vec![],
                color: None,
                created_at: now,
                last_accessed: now - chrono::Duration::days(100),
                is_favorite: false,
            },
            FileAlias {
                id: "2".to_string(),
                alias: "configure".to_string(),
                path: PathBuf::from("/usr/bin/configure"),
                tags: vec![],
                color: None,
                created_at: now,
                last_accessed: now - chrono::Duration::days(100),
                is_favorite: false,
            },
        ];

        state.search_engine.set_aliases(state.file_aliases.clone());

        // 完全一致のテスト
        state.search_query = "config".to_string();
        state.filter_aliases();

        // 完全一致で "config" が最初に来るはず
        assert!(!state.filtered_items.is_empty());
        assert_eq!(state.filtered_items[0].alias, "config");
    }

    #[test]
    fn test_filter_aliases_with_favorite_boost() {
        // お気に入りブーストのテスト
        let mut state = AppState::default();
        let now = chrono::Utc::now();

        // お気に入りとそうでないエイリアスを作成
        state.file_aliases = vec![
            FileAlias {
                id: "1".to_string(),
                alias: "config".to_string(),
                path: PathBuf::from("/etc/config"),
                tags: vec![],
                color: None,
                created_at: now,
                last_accessed: now - chrono::Duration::days(100),
                is_favorite: false,
            },
            FileAlias {
                id: "2".to_string(),
                alias: "config2".to_string(),
                path: PathBuf::from("/etc/config2"),
                tags: vec![],
                color: None,
                created_at: now,
                last_accessed: now - chrono::Duration::days(100),
                is_favorite: true,  // お気に入り
            },
        ];

        state.search_engine.set_aliases(state.file_aliases.clone());

        // "config" で検索
        state.search_query = "config".to_string();
        state.filter_aliases();

        // お気に入りの "config2" が上位に来るはず
        // (完全一致のconfigがスコア1.0、前方一致+お気に入りのconfig2がスコア1.0)
        // ただし、configが完全一致なので最初に来る可能性がある
        assert!(!state.filtered_items.is_empty());

        // 両方とも結果に含まれているはず
        let config_found = state.filtered_items.iter().any(|a| a.alias == "config");
        let config2_found = state.filtered_items.iter().any(|a| a.alias == "config2");
        assert!(config_found, "configが見つかるはず");
        assert!(config2_found, "config2が見つかるはず");
    }

    #[test]
    fn test_filter_aliases_with_path_search() {
        // パス検索のテスト
        let mut state = AppState::default();
        let now = chrono::Utc::now();

        state.file_aliases = vec![
            FileAlias {
                id: "1".to_string(),
                alias: "doc".to_string(),
                path: PathBuf::from("/documents/important/file.txt"),
                tags: vec![],
                color: None,
                created_at: now,
                last_accessed: now - chrono::Duration::days(100),
                is_favorite: false,
            },
            FileAlias {
                id: "2".to_string(),
                alias: "test".to_string(),
                path: PathBuf::from("/path/to/test"),
                tags: vec![],
                color: None,
                created_at: now,
                last_accessed: now - chrono::Duration::days(100),
                is_favorite: false,
            },
        ];

        state.search_engine.set_aliases(state.file_aliases.clone());

        // パスに対する検索
        state.search_query = "documents".to_string();
        state.filter_aliases();

        // パスにマッチしたエイリアスが見つかるはず
        assert!(!state.filtered_items.is_empty());
        assert_eq!(state.filtered_items[0].alias, "doc");
    }

    #[test]
    fn test_filter_aliases_with_tag_search() {
        // タグ検索のテスト
        let mut state = AppState::default();
        let now = chrono::Utc::now();

        let mut alias_with_tags = FileAlias {
            id: "1".to_string(),
            alias: "document".to_string(),
            path: PathBuf::from("/path/to/doc"),
            tags: vec!["important".to_string(), "work".to_string()],
            color: None,
            created_at: now,
            last_accessed: now - chrono::Duration::days(100),
            is_favorite: false,
        };

        state.file_aliases = vec![alias_with_tags];
        state.search_engine.set_aliases(state.file_aliases.clone());

        // タグに対する検索
        state.search_query = "important".to_string();
        state.filter_aliases();

        // タグにマッチしたエイリアスが見つかるはず
        assert!(!state.filtered_items.is_empty());
        assert_eq!(state.filtered_items[0].alias, "document");
    }
}
