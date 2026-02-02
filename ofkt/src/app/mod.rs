pub mod state;

use state::{AppState, BrowseMode, FocusArea};
use eframe::egui;
use log::info;
use crate::ui::theme::Theme;
use crate::ui::search_bar::SearchBar;
use crate::ui::file_tree::FileTreeView;
use crate::ui::context_menu::{ContextMenu, MenuAction};
use crate::core::file_manager::FileManager;
use crate::platform::{theme_detector, TrayEvent};
use crate::utils::path::paths_equal;

/// Ofkt アプリケーション
pub struct OfktApp {
    state: AppState,
    search_bar: SearchBar,
    file_tree: FileTreeView,
}

impl Default for OfktApp {
    fn default() -> Self {
        Self::new()
    }
}

impl OfktApp {
    /// 新しい OfktApp を作成
    ///
    /// # パフォーマンス最適化（Task 6.1.3）
    /// - 起動時は最小限の初期化のみを行う
    /// - 設定とエイリアスの読み込みは遅延初期化で行う
    /// - UI の表示を優先し、起動時間を短縮
    pub fn new() -> Self {
        let state = AppState::new();

        // 起動時は最小限の初期化のみ
        // 設定とエイリアスの読み込みは update() で遅延実行

        Self {
            state,
            search_bar: SearchBar::new(),
            file_tree: FileTreeView::new(),
        }
    }

    /// テーマを適用
    fn apply_theme(&mut self, ctx: &egui::Context) {
        let theme = if let Some(ref config) = self.state.config {
            match config.theme.mode.as_str() {
                "system" => {
                    // システムテーマを検出
                    theme_detector::detect_system_theme()
                }
                "light" => Theme::Light,
                "dark" => Theme::Dark,
                _ => Theme::Dark, // デフォルトはダーク
            }
        } else {
            Theme::Dark
        };

        // テーマを状態に保存
        self.state.current_theme = theme;

        // egui にテーマを適用
        ctx.set_visuals(theme.to_visuals());
    }

    /// ウィンドウの表示/非表示を切り替える
    fn toggle_window_visibility(&mut self, ctx: &egui::Context) {
        self.state.is_window_visible = !self.state.is_window_visible;

        if self.state.is_window_visible {
            // ウィンドウを表示（最小化を解除）
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        } else {
            // ウィンドウを非表示（最小化）
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }

        log::info!("ウィンドウ表示切り替え: {}",
            if self.state.is_window_visible { "表示" } else { "非表示" });
    }

    /// クリップボードからファイルをペースト（ディレクトリモード用）
    fn handle_paste(&mut self) {
        let current_dir = if let Some(ref browser) = self.state.directory_browser {
            browser.current_path().to_path_buf()
        } else {
            log::error!("ディレクトリブラウザが初期化されていません");
            return;
        };

        self.handle_paste_to_dir(current_dir);

        // ディレクトリをリロード
        if let Some(ref mut browser) = self.state.directory_browser {
            if let Err(e) = browser.reload() {
                log::error!("ディレクトリリロード失敗: {}", e);
            }
        }
    }

    /// 指定ディレクトリにクリップボードからファイルをペースト
    fn handle_paste_to_dir(&mut self, dest_dir: std::path::PathBuf) {
        log::info!("ペースト開始: dest_dir={}", dest_dir.display());

        let _file_manager = FileManager::new();
        let paths = self.state.clipboard_state.paths.clone();
        let mode = self.state.clipboard_state.mode;

        log::debug!("クリップボード内容: {} 個のパス, モード={:?}", paths.len(), mode);

        // === 事前検証フェーズ ===
        log::debug!("=== 事前検証フェーズ開始 ===");
        let mut validation_errors = Vec::new();

        // 1. コピー元の存在確認
        for src_path in paths.iter() {
            if !src_path.exists() {
                log::debug!("コピー元存在確認: NG - {}", src_path.display());
                validation_errors.push(format!("「{}」が存在しません",
                    src_path.file_name().unwrap_or_default().to_string_lossy()));
            } else {
                log::debug!("コピー元存在確認: OK - {}", src_path.display());
            }
        }

        // 2. コピー先ディレクトリの確認
        if !dest_dir.exists() {
            log::debug!("コピー先ディレクトリ確認: NG - 存在しない: {}", dest_dir.display());
            validation_errors.push(format!("コピー先ディレクトリ「{}」が存在しません", dest_dir.display()));
        } else if !dest_dir.is_dir() {
            log::debug!("コピー先ディレクトリ確認: NG - ディレクトリではない: {}", dest_dir.display());
            validation_errors.push(format!("「{}」はディレクトリではありません", dest_dir.display()));
        } else {
            log::debug!("コピー先ディレクトリ確認: OK - {}", dest_dir.display());
        }

        // 3. 書き込み権限の確認
        // Windows互換性のため、readonly()チェックをスキップし、実行時エラーで判定
        log::debug!("書き込み権限確認: スキップ（Windows互換性のため実行時チェック）");

        // 4. ディスク容量の推定確認（簡易版）
        // 注: 正確な実装はfs2クレートなどが必要
        log::debug!("ディスク容量確認: スキップ（未実装）");

        // 検証エラーがある場合は警告を表示して中断
        if !validation_errors.is_empty() {
            log::warn!("=== 事前検証フェーズ失敗 === エラー数: {}", validation_errors.len());
            log::warn!("検証エラー: {}", validation_errors.join(", "));
            let error_message = format!("ペースト操作を実行できません:\n{}", validation_errors.join("\n"));
            self.state.paste_result_message = Some(
                crate::app::state::PasteResultMessage::new(
                    error_message,
                    crate::app::state::MessageType::Error
                )
            );
            return;
        }

        log::debug!("=== 事前検証フェーズ完了 ===");

        // ペースト前に上書きされるファイルをチェック
        let mut files_to_overwrite = Vec::new();

        for src_path in paths.iter() {
            if let Some(file_name) = src_path.file_name() {
                let dest_path = dest_dir.join(file_name);
                if dest_path.exists() && src_path != &dest_path {
                    files_to_overwrite.push(dest_path);
                }
            }
        }

        // 上書き対象がある場合、確認ダイアログを表示
        if !files_to_overwrite.is_empty() {
            log::info!("上書き確認ダイアログ表示: {} 個のファイルが上書き対象", files_to_overwrite.len());
            self.state.overwrite_confirmation_dialog = Some(
                crate::app::state::OverwriteConfirmationDialog {
                    files: files_to_overwrite,
                    pending_paste: crate::app::state::PendingPasteOperation {
                        src_paths: paths.clone(),
                        dest_dir: dest_dir.clone(),
                        mode,
                    },
                }
            );
            return; // 確認待ちで処理を保留
        }

        // === 実行フェーズ ===
        // 上書き確認をスキップして実行
        log::info!("ペースト実行（上書き確認なし）");
        self.execute_paste_operation(crate::app::state::PendingPasteOperation {
            src_paths: paths,
            dest_dir,
            mode,
        });
    }

    /// ペースト操作を実行（上書き確認をスキップ）
    fn execute_paste_operation(&mut self, operation: crate::app::state::PendingPasteOperation) {
        use crate::core::clipboard::{ClipboardMode, generate_copy_name};

        let file_manager = FileManager::new();
        let paths = operation.src_paths;
        let dest_dir = operation.dest_dir;
        let mode = operation.mode;

        log::info!("=== ペースト実行開始 === モード: {:?}, ファイル数: {}, 宛先: {}",
            mode, paths.len(), dest_dir.display());

        let mut pasted_paths = Vec::new();
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        for (idx, src_path) in paths.iter().enumerate() {
            log::debug!("[{}/{}] 処理開始: {}", idx + 1, paths.len(), src_path.display());
            let file_name = match src_path.file_name() {
                Some(name) => name,
                None => {
                    log::error!("ファイル名の取得に失敗: {}", src_path.display());
                    error_count += 1;
                    errors.push(format!("ファイル名の取得に失敗: {}", src_path.display()));
                    continue;
                }
            };

            let mut dest_path = dest_dir.join(file_name);

            if src_path == &dest_path {
                dest_path = generate_copy_name(src_path, &dest_dir);
            }

            if dest_path.exists() && src_path != &dest_path {
                log::warn!("「{}」は既に存在します。上書きします。", file_name.to_string_lossy());
            }

            let file_size = src_path.metadata()
                .map(|m| m.len())
                .unwrap_or(0);
            let start_time = std::time::Instant::now();

            match mode {
                ClipboardMode::Copy => {
                    log::debug!("コピー開始: {} -> {} (サイズ: {} bytes)",
                        src_path.display(), dest_path.display(), file_size);
                    if let Err(e) = file_manager.copy_recursive(src_path, &dest_path) {
                        let elapsed = start_time.elapsed();
                        log::error!("コピー失敗: {} (経過時間: {:?})", e, elapsed);
                        error_count += 1;
                        errors.push(format!("「{}」のコピーに失敗: {}", file_name.to_string_lossy(), e));
                    } else {
                        let elapsed = start_time.elapsed();
                        log::info!("「{}」をコピーしました (サイズ: {} bytes, 時間: {:?})",
                            file_name.to_string_lossy(), file_size, elapsed);
                        pasted_paths.push(dest_path.clone());
                        success_count += 1;
                    }
                }
                ClipboardMode::Cut => {
                    log::debug!("移動開始: {} -> {} (サイズ: {} bytes)",
                        src_path.display(), dest_path.display(), file_size);
                    if let Err(e) = file_manager.move_file(src_path, &dest_path) {
                        let elapsed = start_time.elapsed();
                        log::error!("移動失敗: {} (経過時間: {:?})", e, elapsed);
                        error_count += 1;
                        errors.push(format!("「{}」の移動に失敗: {}", file_name.to_string_lossy(), e));
                    } else {
                        let elapsed = start_time.elapsed();
                        log::info!("「{}」を移動しました (サイズ: {} bytes, 時間: {:?})",
                            file_name.to_string_lossy(), file_size, elapsed);
                        pasted_paths.push(dest_path.clone());
                        success_count += 1;
                    }
                }
            }
        }

        // 切り取りモードで全て成功した場合のみクリップボードをクリア
        if mode == ClipboardMode::Cut {
            if error_count == 0 {
                log::info!("Cutモード: 全てのファイル移動が成功したため、クリップボードをクリア");
                self.state.clipboard_state.clear();
            } else {
                log::warn!("Cutモード: {}個のファイル移動に失敗したため、クリップボードを保持", error_count);
            }
        }

        log::info!("=== ペースト実行完了 === 成功: {}, 失敗: {}", success_count, error_count);

        // ペーストハイライトを設定
        if !pasted_paths.is_empty() {
            self.state.pasted_files_highlight = Some(crate::app::state::PastedFileHighlight::new(pasted_paths));
            log::debug!("{}個のファイルをハイライト対象に設定しました", success_count);
        }

        // 結果メッセージを設定
        let message = if error_count == 0 {
            format!("{}個のファイルを{}しました", success_count, if mode == ClipboardMode::Copy { "コピー" } else { "移動" })
        } else if success_count == 0 {
            format!("すべてのファイルの{}に失敗しました:\n{}", if mode == ClipboardMode::Copy { "コピー" } else { "移動" }, errors.join("\n"))
        } else {
            format!("{}個のファイルを{}しましたが、{}個のファイルに失敗しました:\n{}",
                success_count, if mode == ClipboardMode::Copy { "コピー" } else { "移動" }, error_count, errors.join("\n"))
        };

        let message_type = if error_count == 0 {
            crate::app::state::MessageType::Success
        } else if success_count == 0 {
            crate::app::state::MessageType::Error
        } else {
            crate::app::state::MessageType::Warning
        };

        self.state.paste_result_message = Some(crate::app::state::PasteResultMessage::new(message, message_type));
    }

    /// 削除処理を実行するヘルパーメソッド
    ///
    /// # 引数
    /// * `paths` - 削除対象のパス一覧
    /// * `permanent` - true: 完全削除、false: ゴミ箱に移動
    fn execute_delete(&mut self, paths: &[std::path::PathBuf], permanent: bool) {
        let file_manager = FileManager::new();
        let mut success_count = 0;
        let mut errors = Vec::new();

        for path in paths {
            if let Err(e) = file_manager.delete(path, permanent) {
                log::error!("削除に失敗: {}", e);
                errors.push(format!("{}: {}", path.file_name().unwrap_or_default().to_string_lossy(), e));
            } else {
                success_count += 1;
            }
        }

        self.state.delete_confirmation_dialog = None;

        // ディレクトリをリロード
        if let Some(ref mut browser) = self.state.directory_browser {
            let _ = browser.reload();
        }

        // 結果メッセージを設定
        let action = if permanent { "完全に削除" } else { "ゴミ箱に移動" };
        if errors.is_empty() {
            self.state.operation_result_message = Some(
                crate::app::state::OperationResultMessage::success(
                    format!("{} 個のアイテムを{}しました", success_count, action)
                )
            );
        } else {
            self.state.operation_result_message = Some(
                crate::app::state::OperationResultMessage::error(
                    format!("削除に失敗: {}", errors.join(", "))
                )
            );
        }
    }
}

impl eframe::App for OfktApp {
    /// UIの更新
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // フレーム更新ログ（1秒ごとにカウンター表示）
        use std::time::{Duration, Instant};
        use std::sync::Mutex;
        use lazy_static::lazy_static;

        lazy_static! {
            static ref LAST_LOG_TIME: Mutex<Option<Instant>> = Mutex::new(None);
            static ref FRAME_COUNT: Mutex<u64> = Mutex::new(0);
        }

        {
            let mut count = FRAME_COUNT.lock().unwrap();
            *count += 1;

            let mut last_time = LAST_LOG_TIME.lock().unwrap();
            let now = Instant::now();

            if last_time.is_none() || now.duration_since(last_time.unwrap()) >= Duration::from_secs(1) {
                log::info!("Frame update count: {} frames", *count);
                *last_time = Some(now);
                *count = 0;
            }
        }

        // Ctrl+C/X/V の検出
        // ファイルが選択されている場合はファイル操作を優先
        let has_file_selection = match self.state.browse_mode {
            BrowseMode::Alias => self.state.selected_index.is_some(),
            BrowseMode::Directory => self.state.selected_directory_index.is_some(),
        };

        // egui::Eventを直接チェックする方式（Windows互換性のため）
        let mut copy_pressed = false;
        let mut cut_pressed = false;
        let mut paste_pressed = false;

        ctx.input(|i| {
            for event in &i.events {
                match event {
                    egui::Event::Key { key, pressed: true, modifiers, .. } => {
                        if modifiers.ctrl {
                            match key {
                                egui::Key::C => copy_pressed = true,
                                egui::Key::X => cut_pressed = true,
                                egui::Key::V => paste_pressed = true,
                                _ => {}
                            }
                        }
                    }
                    egui::Event::Copy => copy_pressed = true,
                    egui::Event::Cut => cut_pressed = true,
                    egui::Event::Paste(_) => paste_pressed = true,
                    _ => {}
                }
            }
        });

        if copy_pressed {
            log::debug!("[KEYBOARD] Copy event detected (browse_mode={:?}, has_selection={})", self.state.browse_mode, has_file_selection);
        }
        if cut_pressed {
            log::debug!("[KEYBOARD] Cut event detected (browse_mode={:?}, has_selection={})", self.state.browse_mode, has_file_selection);
        }
        if paste_pressed {
            log::debug!("[KEYBOARD] Paste event detected (browse_mode={:?}, has_selection={})", self.state.browse_mode, has_file_selection);
        }

        if copy_pressed && has_file_selection {
            log::info!("[KEYBOARD] Ctrl+C detected! (browse_mode={:?})", self.state.browse_mode);
            self.state.pending_file_copy = true;
        }
        if cut_pressed && has_file_selection {
            log::info!("[KEYBOARD] Ctrl+X detected! (browse_mode={:?})", self.state.browse_mode);
            self.state.pending_file_cut = true;
        }
        if paste_pressed {
            log::info!("[KEYBOARD] Ctrl+V detected! (browse_mode={:?})", self.state.browse_mode);
            self.state.pending_file_paste = true;
        }

        // Ctrl+Z: Undo
        let undo_pressed = ctx.input(|i| {
            i.events.iter().any(|e| {
                matches!(e, egui::Event::Key { key: egui::Key::Z, pressed: true, modifiers, .. } if modifiers.ctrl && !modifiers.shift)
            })
        });

        // Ctrl+Y または Ctrl+Shift+Z: Redo
        let redo_pressed = ctx.input(|i| {
            i.events.iter().any(|e| {
                matches!(e, egui::Event::Key { key: egui::Key::Y, pressed: true, modifiers, .. } if modifiers.ctrl)
                    || matches!(e, egui::Event::Key { key: egui::Key::Z, pressed: true, modifiers, .. } if modifiers.ctrl && modifiers.shift)
            })
        });

        if undo_pressed {
            match self.state.operation_history.undo() {
                Ok(msg) => {
                    self.state.operation_result_message = Some(
                        crate::app::state::OperationResultMessage::success(msg)
                    );
                    // ディレクトリをリロード
                    if let Some(ref mut browser) = self.state.directory_browser {
                        let _ = browser.reload();
                    }
                }
                Err(msg) => {
                    self.state.operation_result_message = Some(
                        crate::app::state::OperationResultMessage::warning(msg)
                    );
                }
            }
        }

        if redo_pressed {
            match self.state.operation_history.redo() {
                Ok(msg) => {
                    self.state.operation_result_message = Some(
                        crate::app::state::OperationResultMessage::success(msg)
                    );
                    if let Some(ref mut browser) = self.state.directory_browser {
                        let _ = browser.reload();
                    }
                }
                Err(msg) => {
                    self.state.operation_result_message = Some(
                        crate::app::state::OperationResultMessage::warning(msg)
                    );
                }
            }
        }

        // ペーストハイライトの期限チェック
        if let Some(ref highlight) = self.state.pasted_files_highlight {
            if highlight.is_expired() {
                self.state.pasted_files_highlight = None;
                log::debug!("ペーストハイライトがタイムアウトしました");
            }
        }

        // ユーザー操作によるクリア
        if self.state.pasted_files_highlight.is_some() {
            // 任意のキー押下でクリア
            let any_key_pressed = ctx.input(|i| {
                i.key_pressed(egui::Key::ArrowUp)
                    || i.key_pressed(egui::Key::ArrowDown)
                    || i.key_pressed(egui::Key::ArrowLeft)
                    || i.key_pressed(egui::Key::ArrowRight)
                    || i.key_pressed(egui::Key::Enter)
                    || i.key_pressed(egui::Key::Escape)
                    || i.key_pressed(egui::Key::Tab)
                    || i.key_pressed(egui::Key::Backspace)
            });

            if any_key_pressed {
                self.state.pasted_files_highlight = None;
                log::debug!("キー操作によりペーストハイライトをクリアしました");
            }

            // マウスクリックでクリア
            if ctx.input(|i| i.pointer.any_click()) {
                self.state.pasted_files_highlight = None;
                log::debug!("マウスクリックによりペーストハイライトをクリアしました");
            }
        }

        // 遅延初期化（初回のみ実行）
        if !self.state.is_initialized() {
            if let Err(e) = self.state.lazy_initialize() {
                log::error!("遅延初期化に失敗: {}", e);
            }
        }

        // テーマを適用
        self.apply_theme(ctx);

        // グローバルホットキーイベントをポーリング（HotkeyManagerが利用可能な場合のみ）
        let hotkey_pressed = self.state.hotkey_manager
            .as_ref()
            .map(|m| m.handle_events())
            .unwrap_or(false);

        if hotkey_pressed {
            // イベント重複防止: 200ms以内の連続イベントを無視
            let now = Instant::now();
            let should_toggle = if let Some(last_time) = self.state.last_hotkey_time {
                now.duration_since(last_time) > Duration::from_millis(200)
            } else {
                true
            };

            if should_toggle {
                self.state.last_hotkey_time = Some(now);
                log::info!("ホットキーが押されました: Ctrl+Shift+O");
                self.toggle_window_visibility(ctx);
            } else {
                log::debug!("ホットキーイベントを重複として無視しました");
            }
        }

        // システムトレイイベントをポーリング
        if let Some(tray_event) = self.state.system_tray.handle_events() {
            match tray_event {
                TrayEvent::Open => {
                    self.toggle_window_visibility(ctx);
                }
                TrayEvent::Settings => {
                    log::info!("トレイメニュー「設定」が選択されました");
                    // TODO: 設定画面を開く（将来実装）
                }
                TrayEvent::Exit => {
                    log::info!("トレイメニュー「終了」が選択されました");
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }

        // Ctrl+Tabでエイリアス/ディレクトリモード切り替え
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Tab)) {
            self.state.browse_mode = match self.state.browse_mode {
                BrowseMode::Alias => BrowseMode::Directory,
                BrowseMode::Directory => BrowseMode::Alias,
            };

            // モード切り替え時にフォーカスをメインパネルに設定
            self.state.current_focus_area = FocusArea::Main;

            log::info!("モード切り替え: {:?}", self.state.browse_mode);
        }

        // ディレクトリモードに切り替えた時、DirectoryBrowserを初期化
        if self.state.browse_mode == BrowseMode::Directory && self.state.directory_browser.is_none() {
            if let Some(home_dir) = dirs::home_dir() {
                if let Err(e) = self.state.init_directory_browser(home_dir) {
                    log::error!("DirectoryBrowserの初期化に失敗: {}", e);
                }
            } else {
                log::warn!("ホームディレクトリの取得に失敗");
            }
        }

        // 共通のトップバー（タブバー）
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.heading("Ofkt - ファイル管理ツール");

            ui.separator();

            // モード切替タブバー
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.state.browse_mode, BrowseMode::Alias, "エイリアス");
                ui.selectable_value(&mut self.state.browse_mode, BrowseMode::Directory, "ディレクトリ");
            });
        });

        // モードに応じたUI表示
        match self.state.browse_mode {
            BrowseMode::Alias => {
                // エイリアスモードUI
                let mut central_panel = egui::CentralPanel::default();

                // メインパネルにフォーカスがある場合は枠線を表示
                if self.state.current_focus_area == FocusArea::Main {
                    central_panel = central_panel.frame(egui::Frame {
                        stroke: egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),  // 青色の枠線
                        ..Default::default()
                    });
                }

                central_panel.show(ctx, |ui| {
                    // Tabキーでフォーカス領域を切り替え（Ctrlなし）
                    if ctx.input(|i| i.key_pressed(egui::Key::Tab) && !i.modifiers.shift && !i.modifiers.ctrl) {
                        self.state.current_focus_area = match self.state.current_focus_area {
                            FocusArea::Search => FocusArea::Sidebar,
                            FocusArea::Sidebar => FocusArea::Main,
                            FocusArea::Main => FocusArea::Search,
                        };

                        // 検索バーにフォーカスする場合はrequest_focus
                        if self.state.current_focus_area == FocusArea::Search {
                            self.search_bar.request_focus(ui.ctx());
                        }
                    }

                    // Shift+Tabで逆方向に切り替え（Ctrlなし）
                    if ctx.input(|i| i.key_pressed(egui::Key::Tab) && i.modifiers.shift && !i.modifiers.ctrl) {
                        self.state.current_focus_area = match self.state.current_focus_area {
                            FocusArea::Search => FocusArea::Main,
                            FocusArea::Main => FocusArea::Sidebar,
                            FocusArea::Sidebar => FocusArea::Search,
                        };

                        if self.state.current_focus_area == FocusArea::Search {
                            self.search_bar.request_focus(ui.ctx());
                        }
                    }

                    // Ctrl+Fで検索バーにフォーカス
                    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::F)) {
                        self.search_bar.request_focus(ui.ctx());
                    }

                    // 検索バー（エイリアス用）
                    let search_event = self.search_bar.render(ui, &mut self.state.search_query);

                    // フォーカス状態を更新
                    self.state.search_bar_focused = search_event.has_focus;

                    // 検索バーにフォーカスがある場合はFocusAreaを更新
                    if search_event.has_focus {
                        self.state.current_focus_area = FocusArea::Search;
                    }

                    if search_event.changed {
                        if self.state.search_debouncer.should_search(&self.state.search_query) {
                            self.state.filter_aliases();
                        }
                    }

                    if search_event.cleared {
                        // 検索がクリアされた場合は即座に全件表示
                        self.state.filter_aliases();
                    }

                    if search_event.submitted {
                        // Enterキーで即座に検索実行（デバウンスをバイパス）
                        self.state.search_debouncer.force_search();
                        self.state.filter_aliases();
                    }

                    // 検索バーで↓キーを押すと、最初の結果を選択
                    if !self.state.filtered_items.is_empty()
                        && self.state.selected_index.is_none()
                        && ui.input(|i| i.key_pressed(egui::Key::ArrowDown))
                    {
                        self.state.selected_index = Some(0);
                    }

                    ui.separator();

                    // 検索結果カウント
                    let total_count = self.state.file_aliases.len();
                    let filtered_count = self.state.filtered_items.len();

                    if self.state.search_query.is_empty() {
                        ui.label(format!("エイリアス: {} 件", total_count));
                    } else {
                        ui.label(format!("検索結果: {} / {} 件", filtered_count, total_count));
                    }

                    ui.separator();

                    // エイリアス追加ボタン
                    if ui.button("+ エイリアス追加").clicked() {
                        self.state.show_add_alias_dialog = true;
                        self.state.new_alias_name.clear();
                        self.state.new_alias_path.clear();
                    }

                    ui.separator();

                    // スクロール可能なエリアでファイルツリーを表示
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            // ファイルツリー
                            // メインパネルにフォーカスがある場合のみハイライト表示
                            let display_selected_index = if self.state.current_focus_area == FocusArea::Main {
                                self.state.selected_index
                            } else {
                                None
                            };

                            let (selected_index, open_index) = self.file_tree.render(
                                ui,
                                &self.state.filtered_items,
                                display_selected_index,
                            );

                            // シングルクリック → 選択のみ
                            if let Some(idx) = selected_index {
                                self.state.selected_index = Some(idx);
                            }

                            // ダブルクリック → ファイルを開く / ディレクトリに移動
                            if let Some(idx) = open_index {
                                self.state.selected_index = Some(idx);

                                if let Some(alias) = self.state.filtered_items.get(idx) {
                                    if alias.path.is_dir() {
                                        if let Err(e) = self.state.init_directory_browser(alias.path.clone()) {
                                            log::error!("エイリアスパスへの移動に失敗: {}", e);
                                        } else {
                                            self.state.browse_mode = BrowseMode::Directory;
                                            // 検索バーをクリア
                                            self.state.search_query.clear();
                                            self.state.selected_index = None;
                                        }
                                    } else {
                                        let file_manager = FileManager::new();
                                        if let Err(e) = file_manager.open(&alias.path) {
                                            log::error!("ファイルを開けませんでした: {}", e);
                                        }
                                    }
                                }
                            }

                            // クリック時のメニュー表示
                            if self.state.selected_index.is_some() {
                                // 右クリックでコンテキストメニューを表示
                                ui.menu_button("操作", |ui| {
                                    if ui.button("削除").clicked() {
                                        // 選択されたエイリアスを削除
                                        if let Some(idx) = self.state.selected_index {
                                            if let Some(alias) = self.state.filtered_items.get(idx) {
                                                let alias_id = alias.id.clone();
                                                let alias_name = alias.alias.clone();

                                                match self.state.alias_manager.remove_alias_by_id(&alias_id) {
                                                    Ok(()) => {
                                                        // 保存
                                                        if let Err(e) = self.state.alias_manager.save() {
                                                            log::error!("エイリアスの保存に失敗: {}", e);
                                                        } else {
                                                            // file_aliasesとfiltered_itemsを更新
                                                            self.state.file_aliases = self.state.alias_manager.get_aliases().to_vec();
                                                            self.state.filter_aliases();
                                                            self.state.selected_index = None;
                                                            log::info!("エイリアス「{}」を削除しました", alias_name);
                                                        }
                                                    }
                                                    Err(e) => {
                                                        log::error!("エイリアスの削除に失敗: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                        ui.close_menu();
                                    }
                                });
                            }
                        });
                });

                // ファイル操作用のキーボードショートカット（Ctrl+C/X/V）
                // pending_file_copy/cut/paste フラグを使用（update()の最初で設定される）

                // Ctrl+C: コピー (pending_file_copyフラグを使用)
                if self.state.pending_file_copy {
                    self.state.pending_file_copy = false;
                    log::info!("[ALIAS] Ctrl+C処理開始 (focus={:?})", self.state.current_focus_area);
                    if let Some(idx) = self.state.selected_index {
                        if let Some(alias) = self.state.filtered_items.get(idx) {
                            self.state.clipboard_state.copy(vec![alias.path.clone()]);
                            log::info!("「{}」をコピーしました", alias.alias);
                            self.state.operation_result_message = Some(
                                crate::app::state::OperationResultMessage::success(
                                    format!("「{}」をコピーしました", alias.alias)
                                )
                            );
                        } else {
                            log::debug!("[ALIAS] selected_index is Some but alias not found");
                        }
                    } else {
                        log::debug!("[ALIAS] selected_index is None");
                    }
                }

                // Ctrl+X: 切り取り (pending_file_cutフラグを使用)
                if self.state.pending_file_cut {
                    self.state.pending_file_cut = false;
                    log::info!("[ALIAS] Ctrl+X処理開始 (focus={:?})", self.state.current_focus_area);
                    if let Some(idx) = self.state.selected_index {
                        if let Some(alias) = self.state.filtered_items.get(idx) {
                            self.state.clipboard_state.cut(vec![alias.path.clone()]);
                            log::info!("「{}」を切り取りました", alias.alias);
                            self.state.operation_result_message = Some(
                                crate::app::state::OperationResultMessage::success(
                                    format!("「{}」を切り取りました", alias.alias)
                                )
                            );
                        }
                    }
                }

                // Ctrl+V: ペースト (pending_file_pasteフラグを使用)
                if self.state.pending_file_paste {
                    self.state.pending_file_paste = false;
                    log::info!("[ALIAS] Ctrl+V処理開始 (focus={:?})", self.state.current_focus_area);
                    if !self.state.clipboard_state.is_empty() {
                        if let Some(home_dir) = dirs::home_dir() {
                            self.handle_paste_to_dir(home_dir);
                        } else {
                            log::error!("[ALIAS] Failed to get home directory");
                        }
                    } else {
                        log::debug!("[ALIAS] clipboard_state is empty");
                    }
                }

                // エイリアスモードのキーイベント処理（統合）

                // メインパネルにフォーカスがある場合のみキーイベント処理を実行
                // ダイアログ表示中はキー入力をスキップ
                if self.state.current_focus_area == FocusArea::Main && !self.state.is_any_dialog_open() {
                    if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                        let max_index = self.state.filtered_items.len().saturating_sub(1);
                        self.state.selected_index = Some(
                            self.state.selected_index
                                .map(|i| (i + 1).min(max_index))
                                .unwrap_or(0)
                        );
                    }

                    if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                        self.state.selected_index = self.state.selected_index
                            .and_then(|i| i.checked_sub(1));
                    }

                    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Some(idx) = self.state.selected_index {
                            if let Some(alias) = self.state.filtered_items.get(idx) {
                                if alias.path.is_dir() {
                                    if let Err(e) = self.state.init_directory_browser(alias.path.clone()) {
                                        log::error!("エイリアスパスへの移動に失敗: {}", e);
                                    } else {
                                        self.state.browse_mode = BrowseMode::Directory;
                                        self.state.search_query.clear();
                                        self.state.selected_index = None;
                                    }
                                } else {
                                    let file_manager = FileManager::new();
                                    if let Err(e) = file_manager.open(&alias.path) {
                                        log::error!("ファイルを開けませんでした: {}", e);
                                    }
                                }
                            }
                        }
                    }

                    // Ctrl+D: クイックアクセスに追加（エイリアスモード）
                    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::D)) {
                        if let Some(idx) = self.state.selected_index {
                            if let Some(alias) = self.state.filtered_items.get(idx) {
                                // 確認ダイアログを表示
                                self.state.add_quick_access_dialog = Some(
                                    crate::app::state::AddQuickAccessDialog::new(
                                        alias.path.clone(),
                                        alias.alias.clone()
                                    )
                                );
                            }
                        }
                    }
                }
            }
            BrowseMode::Directory => {
                // サイドバー
                let mut sidebar_panel = egui::SidePanel::left("drive_panel");

                // サイドバーにフォーカスがある場合は枠線を表示
                if self.state.current_focus_area == FocusArea::Sidebar {
                    sidebar_panel = sidebar_panel.frame(egui::Frame {
                        stroke: egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),  // 青色の枠線
                        ..Default::default()
                    });
                }

                sidebar_panel.show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.heading("場所");
                            ui.separator();

                            // エイリアスセクション
                            ui.label("エイリアス");

                            // お気に入りエイリアスを優先表示
                            let mut aliases = self.state.file_aliases.clone();
                            aliases.sort_by(|a, b| {
                                // お気に入りを優先、その後名前順
                                match (b.is_favorite, a.is_favorite) {
                                    (true, false) => std::cmp::Ordering::Greater,
                                    (false, true) => std::cmp::Ordering::Less,
                                    _ => a.alias.cmp(&b.alias),
                                }
                            });

                            // 検索クエリでフィルタリング
                            let filtered_aliases: Vec<_> = if self.state.directory_search_query.is_empty() {
                                aliases
                            } else {
                                let query = self.state.directory_search_query.to_lowercase();
                                aliases.into_iter()
                                    .filter(|a| a.alias.to_lowercase().contains(&query))
                                    .collect()
                            };

                            // エイリアスリストを表示（最大10件）
                            let displayed_aliases: Vec<_> = filtered_aliases.iter().take(10).collect();
                            let displayed_aliases_count = displayed_aliases.len();

                            for (alias_index, alias) in displayed_aliases.iter().enumerate() {
                                let button_text = if alias.is_favorite {
                                    format!("⭐ {}", alias.alias)
                                } else {
                                    alias.alias.clone()
                                };

                                let button = egui::Button::new(&button_text)
                                    .selected(self.state.current_focus_area == FocusArea::Sidebar
                                        && self.state.selected_sidebar_index == Some(alias_index));

                                if ui.add(button).clicked() {
                                    // エイリアスのパスに移動
                                    if let Err(e) = self.state.init_directory_browser(alias.path.clone()) {
                                        log::error!("エイリアスパスへの移動に失敗: {}", e);
                                    } else {
                                        // 検索バーをクリア
                                        self.state.directory_search_query.clear();
                                        log::info!("エイリアス「{}」を開きました", alias.alias);
                                    }
                                }
                            }

                            ui.separator();

                            // クイックアクセスセクション
                            ui.label("クイックアクセス");

                            // 借用エラーを避けるため、先にclone
                            let quick_access_entries = self.state.quick_access_entries.clone();
                            for (quick_access_index, entry) in quick_access_entries.iter().enumerate() {
                                let sidebar_index = displayed_aliases_count + quick_access_index;

                                let button_text = format!("{}", entry.name);
                                let button = egui::Button::new(&button_text)
                                    .selected(self.state.current_focus_area == FocusArea::Sidebar
                                        && self.state.browse_mode == BrowseMode::Directory
                                        && self.state.selected_sidebar_index == Some(sidebar_index));

                                if ui.add(button).clicked() {
                                    // クリック時の処理
                                    if let Err(e) = self.state.init_directory_browser(entry.path.clone()) {
                                        log::error!("ナビゲーション失敗: {}", e);
                                    } else {
                                        // 検索バーをクリア
                                        self.state.directory_search_query.clear();
                                    }
                                }
                            }

                            ui.separator();

                            // ドライブ
                            ui.label("ドライブ");
                            let drives = crate::platform::get_drives();
                            for (drive_index, drive) in drives.iter().enumerate() {
                                let sidebar_index = displayed_aliases_count + self.state.quick_access_entries.len() + drive_index;

                                let icon = match drive.drive_type {
                                    crate::platform::DriveType::Fixed => "💿",
                                    crate::platform::DriveType::Removable => "💾",
                                    crate::platform::DriveType::Network => "🌐",
                                    _ => "📁",
                                };

                                let button = egui::Button::new(format!("{} {}", icon, drive.name))
                                    .selected(self.state.current_focus_area == FocusArea::Sidebar
                                        && self.state.selected_sidebar_index == Some(sidebar_index));

                                if ui.add(button).clicked() {
                                    if let Err(e) = self.state.init_directory_browser(drive.path.clone()) {
                                        log::error!("ディレクトリブラウザ初期化失敗: {}", e);
                                    } else {
                                        // 検索バーをクリア
                                        self.state.directory_search_query.clear();
                                    }
                                }
                            }

                            ui.separator();

                            // WSL
                            let wsl_dists = crate::platform::get_wsl_distributions();
                            if !wsl_dists.is_empty() {
                                ui.label("WSL");
                                for (wsl_index, dist) in wsl_dists.iter().enumerate() {
                                    let sidebar_index = displayed_aliases_count + self.state.quick_access_entries.len() + drives.len() + wsl_index;

                                    let button = egui::Button::new(format!("🐧 {}", dist.name))
                                        .selected(self.state.current_focus_area == FocusArea::Sidebar
                                            && self.state.selected_sidebar_index == Some(sidebar_index));

                                    if ui.add(button).clicked() {
                                        if let Err(e) = self.state.init_directory_browser(dist.path.clone()) {
                                            log::error!("ディレクトリブラウザ初期化失敗: {}", e);
                                        } else {
                                            // 検索バーをクリア
                                            self.state.directory_search_query.clear();
                                        }
                                    }
                                }
                            }

                            // サイドバーにフォーカスがある場合のキー操作（ctx.inputを使用）
                            if self.state.current_focus_area == FocusArea::Sidebar {
                                // サイドバーの項目数を計算
                                let sidebar_items_count =
                                    displayed_aliases_count  // エイリアスの数
                                    + self.state.quick_access_entries.len()
                                    + drives.len()
                                    + wsl_dists.len();

                                if sidebar_items_count > 0 {
                                    if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                                        let max_index = sidebar_items_count.saturating_sub(1);

                                        match self.state.selected_sidebar_index {
                                            Some(current_index) => {
                                                if current_index >= max_index {
                                                    // 最下部に達したらメインパネルにフォーカス移動
                                                    self.state.current_focus_area = FocusArea::Main;
                                                } else {
                                                    // まだ下に項目があればインデックスを進める
                                                    self.state.selected_sidebar_index = Some(current_index + 1);
                                                }
                                            }
                                            None => {
                                                // 未選択の場合は最初の項目を選択
                                                self.state.selected_sidebar_index = Some(0);
                                            }
                                        }
                                    }

                                    if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                                        self.state.selected_sidebar_index = self.state.selected_sidebar_index
                                            .and_then(|i| i.checked_sub(1));
                                    }

                                    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                                        if let Some(idx) = self.state.selected_sidebar_index {
                                            // インデックスに応じて対応する項目を開く
                                            let mut current_index = 0;

                                            // エイリアスセクション
                                            if idx < displayed_aliases_count {
                                                if let Some(alias) = displayed_aliases.get(idx) {
                                                    if let Err(e) = self.state.init_directory_browser(alias.path.clone()) {
                                                        log::error!("エイリアスパスへの移動に失敗: {}", e);
                                                    } else {
                                                        self.state.directory_search_query.clear();
                                                    }
                                                }
                                            } else {
                                                current_index += displayed_aliases_count;

                                                // クイックアクセスセクション
                                                if idx < current_index + self.state.quick_access_entries.len() {
                                                    let qa_idx = idx - current_index;
                                                    if let Some(entry) = self.state.quick_access_entries.get(qa_idx) {
                                                        if let Err(e) = self.state.init_directory_browser(entry.path.clone()) {
                                                            log::error!("クイックアクセスへの移動に失敗: {}", e);
                                                        } else {
                                                            self.state.directory_search_query.clear();
                                                        }
                                                    }
                                                } else {
                                                    current_index += self.state.quick_access_entries.len();

                                                    // ドライブセクション
                                                    if idx < current_index + drives.len() {
                                                        let drive_idx = idx - current_index;
                                                        if let Some(drive) = drives.get(drive_idx) {
                                                            if let Err(e) = self.state.init_directory_browser(drive.path.clone()) {
                                                                log::error!("ドライブへの移動に失敗: {}", e);
                                                            } else {
                                                                self.state.directory_search_query.clear();
                                                            }
                                                        }
                                                    } else {
                                                        current_index += drives.len();

                                                        // WSLセクション
                                                        if idx < current_index + wsl_dists.len() {
                                                            let wsl_idx = idx - current_index;
                                                            if let Some(dist) = wsl_dists.get(wsl_idx) {
                                                                if let Err(e) = self.state.init_directory_browser(dist.path.clone()) {
                                                                    log::error!("WSL分布版への移動に失敗: {}", e);
                                                                } else {
                                                                    self.state.directory_search_query.clear();
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        });
                });

                // メインパネル
                let mut central_panel = egui::CentralPanel::default();

                // メインパネルにフォーカスがある場合は枠線を表示
                if self.state.current_focus_area == FocusArea::Main {
                    central_panel = central_panel.frame(egui::Frame {
                        stroke: egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),  // 青色の枠線
                        ..Default::default()
                    });
                }

                central_panel.show(ctx, |ui| {
                    // ファイル操作用のキーボードショートカット（Ctrl+C/X/V）
                    // pending_file_copy/cut/paste フラグを使用（update()の最初で設定される）
                    // 重要: これらの処理は directory_browser の有無に関わらずフラグをリセットする必要がある

                    // Ctrl+C: コピー (pending_file_copyフラグを使用)
                    if self.state.pending_file_copy {
                        self.state.pending_file_copy = false;
                        log::info!("[DIRECTORY] Ctrl+C処理開始 (focus={:?})", self.state.current_focus_area);
                        if let Some(ref browser) = self.state.directory_browser {
                            let entries = self.state.get_current_entries();
                            // 検索クエリでフィルタリング
                            let filtered_entries: Vec<_> = if self.state.directory_search_query.is_empty() {
                                entries
                            } else {
                                let query = self.state.directory_search_query.to_lowercase();
                                entries.into_iter()
                                    .filter(|e| e.name.to_lowercase().contains(&query))
                                    .collect()
                            };
                            log::debug!("[DEBUG] selected_directory_index={:?}", self.state.selected_directory_index);
                            if let Some(idx) = self.state.selected_directory_index {
                                if let Some(entry) = filtered_entries.get(idx) {
                                    self.state.clipboard_state.copy(vec![entry.path.clone()]);
                                    log::info!("「{}」をコピーしました", entry.name);
                                    self.state.operation_result_message = Some(
                                        crate::app::state::OperationResultMessage::success(
                                            format!("「{}」をコピーしました", entry.name)
                                        )
                                    );
                                } else {
                                    log::debug!("[DIRECTORY] selected_directory_index is Some but entry not found");
                                }
                            } else {
                                log::debug!("[DIRECTORY] selected_directory_index is None");
                            }
                            let _ = browser; // 借用を明示的に終了
                        } else {
                            log::warn!("[DIRECTORY] Ctrl+C: ディレクトリブラウザが初期化されていません");
                        }
                    }

                    // Ctrl+X: 切り取り (pending_file_cutフラグを使用)
                    if self.state.pending_file_cut {
                        self.state.pending_file_cut = false;
                        log::info!("[DIRECTORY] Ctrl+X処理開始 (focus={:?})", self.state.current_focus_area);
                        if let Some(ref browser) = self.state.directory_browser {
                            let entries = self.state.get_current_entries();
                            // 検索クエリでフィルタリング
                            let filtered_entries: Vec<_> = if self.state.directory_search_query.is_empty() {
                                entries
                            } else {
                                let query = self.state.directory_search_query.to_lowercase();
                                entries.into_iter()
                                    .filter(|e| e.name.to_lowercase().contains(&query))
                                    .collect()
                            };
                            if let Some(idx) = self.state.selected_directory_index {
                                if let Some(entry) = filtered_entries.get(idx) {
                                    self.state.clipboard_state.cut(vec![entry.path.clone()]);
                                    log::info!("「{}」を切り取りました", entry.name);
                                    self.state.operation_result_message = Some(
                                        crate::app::state::OperationResultMessage::success(
                                            format!("「{}」を切り取りました", entry.name)
                                        )
                                    );
                                }
                            }
                            let _ = browser; // 借用を明示的に終了
                        } else {
                            log::warn!("[DIRECTORY] Ctrl+X: ディレクトリブラウザが初期化されていません");
                        }
                    }

                    // Ctrl+V: ペースト (pending_file_pasteフラグを使用)
                    if self.state.pending_file_paste {
                        self.state.pending_file_paste = false;
                        log::info!("[DIRECTORY] Ctrl+V処理開始 (focus={:?})", self.state.current_focus_area);
                        if !self.state.clipboard_state.is_empty() {
                            if self.state.directory_browser.is_some() {
                                self.handle_paste();
                            } else {
                                log::warn!("[DIRECTORY] Ctrl+V: ディレクトリブラウザが初期化されていません");
                            }
                        } else {
                            log::debug!("[DIRECTORY] clipboard_state is empty");
                        }
                    }

                    // Tabキーでフォーカス領域を切り替え（Ctrlなし）
                    // ディレクトリモード: 検索→メイン→サイド
                    if ctx.input(|i| i.key_pressed(egui::Key::Tab) && !i.modifiers.shift && !i.modifiers.ctrl) {
                        self.state.current_focus_area = match self.state.current_focus_area {
                            FocusArea::Search => FocusArea::Main,      // 検索 → メイン
                            FocusArea::Main => FocusArea::Sidebar,     // メイン → サイド
                            FocusArea::Sidebar => FocusArea::Search,   // サイド → 検索
                        };

                        if self.state.current_focus_area == FocusArea::Search {
                            self.search_bar.request_focus(ui.ctx());
                        }
                    }

                    // Shift+Tabで逆方向に切り替え（Ctrlなし）
                    // ディレクトリモード: 検索←メイン←サイド
                    if ctx.input(|i| i.key_pressed(egui::Key::Tab) && i.modifiers.shift && !i.modifiers.ctrl) {
                        self.state.current_focus_area = match self.state.current_focus_area {
                            FocusArea::Search => FocusArea::Sidebar,   // 検索 ← サイド
                            FocusArea::Sidebar => FocusArea::Main,     // サイド ← メイン
                            FocusArea::Main => FocusArea::Search,      // メイン ← 検索
                        };

                        if self.state.current_focus_area == FocusArea::Search {
                            self.search_bar.request_focus(ui.ctx());
                        }
                    }

                    // Ctrl+Fで検索バーにフォーカス
                    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::F)) {
                        self.search_bar.request_focus(ui.ctx());
                    }

                    // 検索バー（ディレクトリ用）
                    let dir_search_event = self.search_bar.render(ui, &mut self.state.directory_search_query);

                    // フォーカス状態を更新
                    self.state.directory_search_bar_focused = dir_search_event.has_focus;

                    // 検索バーにフォーカスがある場合はFocusAreaを更新
                    if dir_search_event.has_focus {
                        self.state.current_focus_area = FocusArea::Search;
                    }

                    if dir_search_event.changed || dir_search_event.cleared || dir_search_event.submitted {
                        // 検索クエリ変更時のログ
                        log::debug!("ディレクトリ検索: {}", self.state.directory_search_query);
                    }

                    ui.separator();

                    if self.state.directory_browser.is_some() {
                        let entries = self.state.get_current_entries();

                        // 検索クエリでフィルタリング
                        let filtered_entries: Vec<_> = if self.state.directory_search_query.is_empty() {
                            entries
                        } else {
                            let query = self.state.directory_search_query.to_lowercase();
                            entries.into_iter()
                                .filter(|e| e.name.to_lowercase().contains(&query))
                                .collect()
                        };

                        // 現在のパス表示
                        let current_path = self.state.directory_browser.as_ref().unwrap().current_path().to_path_buf();
                        ui.label(format!("パス: {}", current_path.display()));

                        // ナビゲーションボタンの状態を取得
                        let can_back = self.state.directory_browser.as_ref().unwrap().can_go_back();
                        let can_forward = self.state.directory_browser.as_ref().unwrap().can_go_forward();

                        // 戻る/進む/親フォルダボタン
                        ui.horizontal(|ui| {
                            if ui.add_enabled(can_back, egui::Button::new("← 戻る")).clicked() {
                                if let Err(e) = self.state.directory_browser.as_mut().unwrap().go_back() {
                                    log::error!("戻るに失敗: {}", e);
                                } else {
                                    // 検索バーをクリア
                                    self.state.directory_search_query.clear();
                                }
                            }
                            if ui.add_enabled(can_forward, egui::Button::new("進む →")).clicked() {
                                if let Err(e) = self.state.directory_browser.as_mut().unwrap().go_forward() {
                                    log::error!("進むに失敗: {}", e);
                                } else {
                                    // 検索バーをクリア
                                    self.state.directory_search_query.clear();
                                }
                            }
                            if ui.button("親フォルダ ↑").clicked() {
                                if let Err(e) = self.state.directory_browser.as_mut().unwrap().parent() {
                                    log::error!("親フォルダへの移動に失敗: {}", e);
                                } else {
                                    // 検索バーをクリア
                                    self.state.directory_search_query.clear();
                                }
                            }
                        });

                        ui.separator();

                        // フィルタリングされたエントリ数を表示
                        ui.label(format!("エントリ: {} 件", filtered_entries.len()));

                        ui.separator();

                        // メインパネルにフォーカスがある場合のみキーイベント処理を実行
                        // ダイアログ表示中はキー入力をスキップ
                        if self.state.current_focus_area == FocusArea::Main && !self.state.is_any_dialog_open() {
                            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                                if let Some(idx) = self.state.selected_directory_index {
                                    if let Some(entry) = filtered_entries.get(idx) {
                                        if entry.is_directory {
                                            // ディレクトリの場合は移動
                                            if let Err(e) = self.state.directory_browser.as_mut().unwrap().navigate_to(entry.path.clone()) {
                                                log::error!("ディレクトリの移動に失敗: {}", e);
                                            } else {
                                                // 検索バーをクリア
                                                self.state.directory_search_query.clear();
                                            }
                                        } else {
                                            // ファイルの場合は開く
                                            let file_manager = FileManager::new();
                                            if let Err(e) = file_manager.open(&entry.path) {
                                                log::error!("ファイルを開くのに失敗: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                            // Backspaceキー（検索バーフォーカス時はスキップ）
                            if !self.state.directory_search_bar_focused
                                && ctx.input(|i| i.key_pressed(egui::Key::Backspace))
                            {
                                if let Err(e) = self.state.directory_browser.as_mut().unwrap().parent() {
                                    log::error!("親フォルダへの移動に失敗: {}", e);
                                } else {
                                    // 検索バーをクリア
                                    self.state.directory_search_query.clear();
                                }
                            }
                            if ctx.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::ArrowLeft)) {
                                if let Err(e) = self.state.directory_browser.as_mut().unwrap().go_back() {
                                    log::error!("戻るに失敗: {}", e);
                                } else {
                                    // 検索バーをクリア
                                    self.state.directory_search_query.clear();
                                }
                            }
                            if ctx.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::ArrowRight)) {
                                if let Err(e) = self.state.directory_browser.as_mut().unwrap().go_forward() {
                                    log::error!("進むに失敗: {}", e);
                                } else {
                                    // 検索バーをクリア
                                    self.state.directory_search_query.clear();
                                }
                            }

                            // 右キー: ディレクトリ展開（Alt+ArrowRightと競合しないようにチェック）
                            if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight) && !i.modifiers.alt) {
                                if let Some(idx) = self.state.selected_directory_index {
                                    if let Some(entry) = filtered_entries.get(idx) {
                                        if entry.is_directory && !self.state.expanded_directories.contains(&entry.path) {
                                            self.state.expanded_directories.insert(entry.path.clone());
                                            log::debug!("ディレクトリ展開: {}", entry.path.display());
                                        }
                                    }
                                }
                            }

                            // 左キー: ディレクトリ折りたたみ/親選択（Alt+ArrowLeftと競合しないようにチェック）
                            if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft) && !i.modifiers.alt) {
                                if let Some(idx) = self.state.selected_directory_index {
                                    if let Some(entry) = filtered_entries.get(idx) {
                                        if entry.is_directory {
                                            if self.state.expanded_directories.contains(&entry.path) {
                                                // 展開されている場合は折りたたみ
                                                self.state.expanded_directories.remove(&entry.path);
                                                log::debug!("ディレクトリ折りたたみ: {}", entry.path.display());
                                            } else {
                                                // 折りたたまれている場合は親ディレクトリを選択
                                                if let Some(parent_path) = entry.path.parent() {
                                                    // 親パスがフィルタに含まれるか確認
                                                    if let Some(parent_idx) = filtered_entries.iter().position(|e| {
                                                        use crate::utils::path::paths_equal;
                                                        paths_equal(&e.path, parent_path)
                                                    }) {
                                                        self.state.selected_directory_index = Some(parent_idx);
                                                        log::debug!("親ディレクトリ選択: {}", parent_path.display());
                                                    } else {
                                                        // 親がフィルタに含まれない場合、検索をクリア
                                                        if !self.state.directory_search_query.is_empty() {
                                                            log::warn!("親ディレクトリがフィルタに含まれていないため、検索をクリアします");
                                                            self.state.directory_search_query.clear();

                                                            // 警告メッセージを表示
                                                            self.state.paste_result_message = Some(
                                                                crate::app::state::PasteResultMessage::new(
                                                                    format!("親ディレクトリ「{}」は検索結果に含まれていないため、検索をクリアしました",
                                                                        parent_path.display()),
                                                                    crate::app::state::MessageType::Warning
                                                                )
                                                            );

                                                            // ディレクトリブラウザをリロードして全エントリを表示
                                                            if let Some(ref mut browser) = self.state.directory_browser {
                                                                if let Err(e) = browser.reload() {
                                                                    log::error!("ディレクトリリロード失敗: {}", e);
                                                                } else {
                                                                    // リロード後、親ディレクトリを検索して選択
                                                                    let entries = browser.entries();
                                                                    if let Some(parent_idx) = entries.iter().position(|e| {
                                                                        use crate::utils::path::paths_equal;
                                                                        paths_equal(&e.path, parent_path)
                                                                    }) {
                                                                        self.state.selected_directory_index = Some(parent_idx);
                                                                    }
                                                                }
                                                            }
                                                        } else {
                                                            // 検索していないのに親が見つからない場合（通常起こらない）
                                                            log::error!("親ディレクトリが見つかりません: {}", parent_path.display());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Ctrl+D: クイックアクセスに追加（確認ダイアログを表示）
                            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::D)) {
                                if let Some(idx) = self.state.selected_directory_index {
                                    if let Some(entry) = filtered_entries.get(idx) {
                                        if entry.is_directory {
                                            // 確認ダイアログを表示
                                            self.state.add_quick_access_dialog = Some(
                                                crate::app::state::AddQuickAccessDialog::new(
                                                    entry.path.clone(),
                                                    entry.name.clone()
                                                )
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        // スクロール可能なエリアでファイルツリーを表示
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                // ファイルツリー表示（filtered_entriesを使用）
                                // メインパネルにフォーカスがある場合のみハイライト表示
                                let display_selected_index = if self.state.current_focus_area == FocusArea::Main {
                                    self.state.selected_directory_index
                                } else {
                                    None
                                };

                                let (selected_path, open_path, is_right_click, total_items) = self.file_tree.render_directory_tree(
                                    ui,
                                    &filtered_entries,
                                    &mut self.state.expanded_directories,
                                    display_selected_index,
                                    self.state.pasted_files_highlight.as_ref()
                                );

                                // キーボードナビゲーション（ArrowDown/ArrowUp）
                                // total_items（展開されたツリー全体）を使用
                                if self.state.current_focus_area == FocusArea::Main {
                                    if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                                        let max_index = total_items.saturating_sub(1);
                                        self.state.selected_directory_index = Some(
                                            self.state.selected_directory_index.map(|i| (i + 1).min(max_index)).unwrap_or(0)
                                        );
                                    }
                                    if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                                        self.state.selected_directory_index = self.state.selected_directory_index.and_then(|i| i.checked_sub(1));
                                    }
                                }

                                // シングルクリック → 選択のみ
                                if let Some(ref path) = selected_path {
                                    // パスからインデックスを検索
                                    self.state.selected_directory_index = filtered_entries.iter()
                                        .position(|e| paths_equal(&e.path, path));

                                    if is_right_click {
                                        // 右クリックの場合、コンテキストメニュー状態を設定
                                        if let Some(entry) = filtered_entries.iter().find(|e| paths_equal(&e.path, path)) {
                                            let pointer_pos = ctx.input(|i| i.pointer.hover_pos().unwrap_or(egui::Pos2::ZERO));
                                            self.state.context_menu_state = Some(
                                                crate::app::state::ContextMenuState::new(
                                                    pointer_pos,
                                                    entry.path.clone(),
                                                    entry.name.clone(),
                                                    entry.is_directory,
                                                )
                                            );
                                        }
                                    }
                                }

                                // ダブルクリック → ファイルを開く / ディレクトリに移動
                                if let Some(ref path) = open_path {
                                    if let Some(entry) = filtered_entries.iter().find(|e| paths_equal(&e.path, path)) {
                                        if entry.is_directory {
                                            // ディレクトリをダブルクリックで移動
                                            if let Err(e) = self.state.directory_browser.as_mut().unwrap().navigate_to(entry.path.clone()) {
                                                log::error!("ディレクトリの移動に失敗: {}", e);
                                            } else {
                                                // 検索バーをクリア
                                                self.state.directory_search_query.clear();
                                            }
                                        } else {
                                            // ファイルをダブルクリックで開く
                                            let file_manager = FileManager::new();
                                            if let Err(e) = file_manager.open(&entry.path) {
                                                log::error!("ファイルを開くのに失敗: {}", e);
                                            }
                                        }
                                    }
                                }
                            });
                    } else {
                        ui.label("ディレクトリブラウザが初期化されていません");
                    }
                });
            }
        }

        // エイリアス追加ダイアログ
        if self.state.show_add_alias_dialog {
            egui::Window::new("エイリアス追加")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("エイリアス名:");
                    ui.text_edit_singleline(&mut self.state.new_alias_name);

                    ui.label("パス:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.state.new_alias_path);
                        if ui.button("...").clicked() {
                            // ディレクトリ選択ダイアログ（将来実装）
                            log::info!("ディレクトリ選択ダイアログ（未実装）");
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("追加").clicked() {
                            // バリデーション
                            if self.state.new_alias_name.is_empty() {
                                log::warn!("エイリアス名が空です");
                            } else if self.state.new_alias_path.is_empty() {
                                log::warn!("パスが空です");
                            } else {
                                // エイリアスを追加
                                match self.state.alias_manager.add_alias(
                                    self.state.new_alias_name.clone(),
                                    std::path::PathBuf::from(&self.state.new_alias_path),
                                    vec![],
                                    None,
                                    false,
                                ) {
                                    Ok(()) => {
                                        // 保存
                                        if let Err(e) = self.state.alias_manager.save() {
                                            log::error!("エイリアスの保存に失敗: {}", e);
                                        } else {
                                            // file_aliasesとfiltered_itemsを更新
                                            self.state.file_aliases = self.state.alias_manager.get_aliases().to_vec();
                                            self.state.filter_aliases();
                                            log::info!("エイリアス「{}」を追加しました", self.state.new_alias_name);
                                            self.state.show_add_alias_dialog = false;
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("エイリアスの追加に失敗: {}", e);
                                    }
                                }
                            }
                        }

                        if ui.button("キャンセル").clicked() {
                            self.state.show_add_alias_dialog = false;
                        }
                    });
                });
        }

        // ペースト結果メッセージの表示
        if let Some(ref msg) = self.state.paste_result_message {
            // 表示から500ms経過後、任意のキー押下で閉じる
            let can_dismiss = msg.timestamp.elapsed() > std::time::Duration::from_millis(500);
            let any_key_pressed = ctx.input(|i| {
                i.key_pressed(egui::Key::Enter)
                    || i.key_pressed(egui::Key::Escape)
                    || i.key_pressed(egui::Key::Space)
                    || i.key_pressed(egui::Key::ArrowUp)
                    || i.key_pressed(egui::Key::ArrowDown)
                    || i.key_pressed(egui::Key::ArrowLeft)
                    || i.key_pressed(egui::Key::ArrowRight)
            });

            if msg.is_expired() || (can_dismiss && any_key_pressed) {
                self.state.paste_result_message = None;
            } else {
                let title = match msg.message_type {
                    crate::app::state::MessageType::Success => "✓ 成功",
                    crate::app::state::MessageType::Error => "✗ エラー",
                    crate::app::state::MessageType::Warning => "⚠ 警告",
                };

                let message_clone = msg.message.clone();
                let mut open = true;
                let mut should_close = false;
                egui::Window::new(title)
                    .open(&mut open)
                    .resizable(false)
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.label(&message_clone);
                        ui.add_space(10.0);
                        if ui.button("OK").clicked() {
                            should_close = true;
                        }
                    });

                if !open || should_close {
                    self.state.paste_result_message = None;
                }
            }
        }

        // 操作結果メッセージの表示
        if let Some(ref msg) = self.state.operation_result_message {
            if msg.is_expired() {
                self.state.operation_result_message = None;
            } else {
                let color = match msg.message_type {
                    crate::app::state::MessageType::Success => egui::Color32::from_rgb(200, 255, 200),
                    crate::app::state::MessageType::Error => egui::Color32::from_rgb(255, 200, 200),
                    crate::app::state::MessageType::Warning => egui::Color32::from_rgb(255, 255, 200),
                };

                let message_clone = msg.message.clone();
                egui::Window::new("操作結果")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_TOP, [0.0, 50.0])
                    .frame(egui::Frame::window(&ctx.style()).fill(color))
                    .show(ctx, |ui| {
                        ui.label(&message_clone);
                    });
            }
        }

        // 上書き確認ダイアログ
        if let Some(ref dialog) = self.state.overwrite_confirmation_dialog {
            log::debug!("上書き確認ダイアログを描画中: {} 個のファイル", dialog.files.len());
            let mut should_close = false;
            let mut should_proceed = false;

            egui::Window::new("⚠ 上書き確認")
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(format!("{}個のファイルが既に存在します。上書きしますか？", dialog.files.len()));
                    ui.add_space(10.0);

                    // ファイル一覧（最大5件表示）
                    for (_i, file) in dialog.files.iter().take(5).enumerate() {
                        ui.label(format!("• {}", file.file_name().unwrap_or_default().to_string_lossy()));
                    }
                    if dialog.files.len() > 5 {
                        ui.label(format!("...他{}個", dialog.files.len() - 5));
                    }

                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("上書きする").clicked() {
                            log::info!("上書き確認: ユーザーが「上書きする」を選択");
                            should_proceed = true;
                            should_close = true;
                        }
                        if ui.button("キャンセル").clicked() {
                            log::info!("上書き確認: ユーザーが「キャンセル」を選択");
                            should_close = true;
                        }
                    });
                });

            if should_proceed {
                log::info!("上書き確認後、ペースト処理を実行");
                let pending = dialog.pending_paste.clone();
                self.state.overwrite_confirmation_dialog = None;
                // 実際のペースト処理を実行（上書きを許可）
                self.execute_paste_operation(pending);

                // ディレクトリをリロード
                if let Some(ref mut browser) = self.state.directory_browser {
                    if let Err(e) = browser.reload() {
                        log::error!("ディレクトリリロード失敗: {}", e);
                    }
                }
            } else if should_close {
                self.state.overwrite_confirmation_dialog = None;
            }
        }

        // クイックアクセス追加確認ダイアログ
        if let Some(ref mut dialog) = self.state.add_quick_access_dialog {
            let mut should_close = false;
            let mut should_add = false;

            egui::Window::new("クイックアクセスに追加")
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("フォルダをクイックアクセスに追加しますか？");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label("名前:");
                        ui.text_edit_singleline(&mut dialog.name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("パス:");
                        ui.label(dialog.path.display().to_string());
                    });

                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("追加").clicked() {
                            should_add = true;
                            should_close = true;
                        }
                        if ui.button("キャンセル").clicked() {
                            should_close = true;
                        }
                    });
                });

            if should_add {
                // クイックアクセスに追加
                let name = dialog.name.clone();
                let path = dialog.path.clone();

                match self.state.add_to_quick_access(name.clone(), path.clone()) {
                    Ok(_) => {
                        log::info!("「{}」をクイックアクセスに追加しました", name);

                        // 成功メッセージを表示
                        self.state.paste_result_message = Some(
                            crate::app::state::PasteResultMessage::new(
                                format!("「{}」をクイックアクセスに追加しました", name),
                                crate::app::state::MessageType::Success
                            )
                        );
                    }
                    Err(e) => {
                        log::error!("クイックアクセスへの追加に失敗: {}", e);

                        // エラーメッセージを表示
                        self.state.paste_result_message = Some(
                            crate::app::state::PasteResultMessage::new(
                                format!("クイックアクセスへの追加に失敗しました: {}", e),
                                crate::app::state::MessageType::Error
                            )
                        );
                    }
                }
            }

            if should_close {
                self.state.add_quick_access_dialog = None;
            }
        }

        // 削除確認ダイアログの表示
        let mut delete_action: Option<bool> = None; // Some(true): 完全削除、Some(false): ゴミ箱
        let mut delete_paths: Vec<std::path::PathBuf> = Vec::new();
        let mut should_cancel_delete = false;

        if let Some(ref dialog) = self.state.delete_confirmation_dialog {
            let dialog_clone = dialog.clone();
            delete_paths = dialog_clone.paths.clone();

            egui::Window::new("削除の確認")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        // 削除対象の表示
                        ui.label("以下を削除しますか？");
                        ui.add_space(8.0);

                        for (i, name) in dialog_clone.display_names.iter().enumerate() {
                            if i < 5 {
                                ui.label(format!("  - {}", name));
                            } else if i == 5 {
                                ui.label(format!("  ...他 {} 個", dialog_clone.display_names.len() - 5));
                                break;
                            }
                        }

                        ui.add_space(16.0);

                        ui.horizontal(|ui| {
                            if ui.button("ゴミ箱に移動").clicked() {
                                delete_action = Some(false);
                            }

                            if ui.button("完全に削除").clicked() {
                                delete_action = Some(true);
                            }

                            if ui.button("キャンセル").clicked() {
                                should_cancel_delete = true;
                            }
                        });
                    });
                });
        }

        // 削除アクションの実行（ダイアログ表示後）
        if let Some(permanent) = delete_action {
            self.execute_delete(&delete_paths, permanent);
        } else if should_cancel_delete {
            self.state.delete_confirmation_dialog = None;
        }

        // リネームダイアログの表示
        if self.state.rename_dialog.is_some() {
            let mut should_close = false;
            let mut should_rename = false;
            let mut new_name = String::new();
            let mut target_path = std::path::PathBuf::new();

            if let Some(ref mut dialog) = self.state.rename_dialog {
                new_name = dialog.new_name.clone();
                target_path = dialog.path.clone();

                egui::Window::new("名前の変更")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label(format!("「{}」の新しい名前:", dialog.original_name));
                        ui.add_space(8.0);

                        let response = ui.text_edit_singleline(&mut dialog.new_name);
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            should_rename = true;
                        }

                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            if ui.button("変更").clicked() {
                                should_rename = true;
                            }
                            if ui.button("キャンセル").clicked() {
                                should_close = true;
                            }
                        });

                        new_name = dialog.new_name.clone();
                    });
            }

            if should_rename && !new_name.is_empty() {
                let original_name = self.state.rename_dialog.as_ref()
                    .map(|d| d.original_name.clone())
                    .unwrap_or_default();
                let new_path = target_path.parent()
                    .map(|p| p.join(&new_name))
                    .unwrap_or_else(|| std::path::PathBuf::from(&new_name));

                if let Err(e) = std::fs::rename(&target_path, &new_path) {
                    log::error!("リネームに失敗: {}", e);
                    self.state.operation_result_message = Some(
                        crate::app::state::OperationResultMessage::error(
                            format!("リネームに失敗: {}", e)
                        )
                    );
                } else {
                    log::info!("リネーム成功: {} -> {}", target_path.display(), new_path.display());
                    // 履歴に追加
                    self.state.operation_history.push(
                        crate::core::operation_history::FileOperation::Rename {
                            old_path: target_path.clone(),
                            new_path: new_path.clone(),
                        }
                    );
                    if let Some(ref mut browser) = self.state.directory_browser {
                        let _ = browser.reload();
                    }
                    self.state.operation_result_message = Some(
                        crate::app::state::OperationResultMessage::success(
                            format!("「{}」を「{}」に変更しました", original_name, new_name)
                        )
                    );
                }
                self.state.rename_dialog = None;
            } else if should_close {
                self.state.rename_dialog = None;
            }
        }

        // プロパティダイアログの表示
        if self.state.properties_dialog.is_some() {
            let mut should_close = false;

            if let Some(ref dialog) = self.state.properties_dialog {
                let dialog_clone = dialog.clone();
                egui::Window::new("プロパティ")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.vertical(|ui| {
                            ui.label(format!("名前: {}", dialog_clone.name));
                            ui.label(format!("種類: {}", if dialog_clone.is_directory { "フォルダ" } else { "ファイル" }));
                            ui.label(format!("サイズ: {} バイト", dialog_clone.size));
                            ui.label(format!("読み取り専用: {}", if dialog_clone.is_readonly { "はい" } else { "いいえ" }));

                            if let Some(modified) = dialog_clone.modified {
                                if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                                    ui.label(format!("更新日時: {:?}", duration));
                                }
                            }

                            ui.add_space(16.0);
                            if ui.button("閉じる").clicked() {
                                should_close = true;
                            }
                        });
                    });
            }

            if should_close {
                self.state.properties_dialog = None;
            }
        }

        // コンテキストメニューの表示
        if self.state.context_menu_state.is_some() {
            let mut should_close = false;
            let mut action_to_execute: Option<MenuAction> = None;
            let mut menu_state_clone: Option<crate::app::state::ContextMenuState> = None;
            let mut menu_rect: Option<egui::Rect> = None;

            if let Some(ref menu_state) = self.state.context_menu_state {
                menu_state_clone = Some(menu_state.clone());

                let area_response = egui::Area::new(egui::Id::new("context_menu"))
                    .fixed_pos(menu_state.position)
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        egui::Frame::popup(ui.style()).show(ui, |ui| {
                            ui.set_min_width(120.0);

                            if ui.button("開く").clicked() {
                                action_to_execute = Some(MenuAction::Open);
                                should_close = true;
                            }
                            ui.separator();
                            if ui.button("コピー").clicked() {
                                action_to_execute = Some(MenuAction::Copy);
                                should_close = true;
                            }
                            if ui.button("切り取り").clicked() {
                                action_to_execute = Some(MenuAction::Cut);
                                should_close = true;
                            }
                            // 貼り付けボタン（クリップボードが空の場合は無効化）
                            if !self.state.clipboard_state.is_empty() {
                                if ui.button("貼り付け").clicked() {
                                    action_to_execute = Some(MenuAction::Paste);
                                    should_close = true;
                                }
                            } else {
                                ui.add_enabled(false, egui::Button::new("貼り付け"));
                            }
                            ui.separator();
                            if ui.button("名前の変更").clicked() {
                                action_to_execute = Some(MenuAction::Rename);
                                should_close = true;
                            }
                            if ui.button("削除").clicked() {
                                action_to_execute = Some(MenuAction::Delete);
                                should_close = true;
                            }
                            ui.separator();
                            if ui.button("プロパティ").clicked() {
                                action_to_execute = Some(MenuAction::Properties);
                                should_close = true;
                            }
                        });
                    });

                menu_rect = Some(area_response.response.rect);
            }

            // メニュー外をクリックしたら閉じる（左クリック時のみ）
            // pointer.primary_released() を使用して、右クリックでメニューを開いた直後に閉じるのを防ぐ
            if ctx.input(|i| i.pointer.primary_released()) {
                if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                    if let Some(rect) = menu_rect {
                        if !rect.contains(pos) {
                            should_close = true;
                        }
                    }
                }
            }

            // Escキーでも閉じる
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                should_close = true;
            }

            // アクションの実行
            if let Some(action) = action_to_execute {
                if let Some(ref menu_state) = menu_state_clone {
                    let file_manager = FileManager::new();
                    match action {
                        MenuAction::Open => {
                            if menu_state.is_directory {
                                if let Some(ref mut browser) = self.state.directory_browser {
                                    let _ = browser.navigate_to(menu_state.entry_path.clone());
                                    self.state.directory_search_query.clear();
                                }
                            } else {
                                let _ = file_manager.open(&menu_state.entry_path);
                            }
                        }
                        MenuAction::Copy => {
                            self.state.clipboard_state.copy(vec![menu_state.entry_path.clone()]);
                            self.state.operation_result_message = Some(
                                crate::app::state::OperationResultMessage::success(
                                    format!("「{}」をコピーしました", menu_state.entry_name)
                                )
                            );
                        }
                        MenuAction::Cut => {
                            self.state.clipboard_state.cut(vec![menu_state.entry_path.clone()]);
                            self.state.operation_result_message = Some(
                                crate::app::state::OperationResultMessage::success(
                                    format!("「{}」を切り取りました", menu_state.entry_name)
                                )
                            );
                        }
                        MenuAction::Paste => {
                            // 現在のディレクトリにペースト
                            self.handle_paste();
                        }
                        MenuAction::Delete => {
                            self.state.delete_confirmation_dialog = Some(
                                crate::app::state::DeleteConfirmationDialog::new(vec![menu_state.entry_path.clone()])
                            );
                        }
                        MenuAction::Rename => {
                            self.state.rename_dialog = Some(
                                crate::app::state::RenameDialog::new(menu_state.entry_path.clone())
                            );
                        }
                        MenuAction::Properties => {
                            self.state.properties_dialog = Some(
                                crate::app::state::PropertiesDialog::new(menu_state.entry_path.clone())
                            );
                        }
                        _ => {}
                    }
                }
            }

            if should_close {
                self.state.context_menu_state = None;
            }
        }

        // 非アクティブ時でもホットキーを検出できるように定期的に再描画をリクエスト
        ctx.request_repaint_after(Duration::from_millis(100));
    }

    /// アプリケーション終了時の保存処理
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        info!("アプリケーション終了");
    }
}
