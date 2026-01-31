use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

/// トレイアイコンのイベント
///
/// ユーザーがトレイメニューから選択したアクションを表します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    /// "開く" メニューが選択された
    Open,
    /// "設定" メニューが選択された
    Settings,
    /// "終了" メニューが選択された
    Exit,
}

/// システムトレイ管理
///
/// トレイアイコンとメニューの管理機能を提供します。
/// 機能:
/// - トレイアイコンの表示
/// - メニューの作成と管理
/// - イベント処理
/// - アイコン状態の切り替え
pub struct SystemTray {
    /// トレイアイコン
    tray_icon: Option<TrayIcon>,
    /// トレイメニュー
    menu: Option<Menu>,
    /// "開く" メニューアイテムのID
    menu_item_open_id: Option<String>,
    /// "設定" メニューアイテムのID
    menu_item_settings_id: Option<String>,
    /// "終了" メニューアイテムのID
    menu_item_exit_id: Option<String>,
}

impl SystemTray {
    /// 新しい SystemTray を作成
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ofkt::platform::SystemTray;
    ///
    /// let tray = SystemTray::new();
    /// ```
    pub fn new() -> Self {
        Self {
            tray_icon: None,
            menu: None,
            menu_item_open_id: None,
            menu_item_settings_id: None,
            menu_item_exit_id: None,
        }
    }

    /// トレイアイコンとメニューを構築
    ///
    /// # Returns
    ///
    /// 成功時は `Ok(())`、失敗時はエラーメッセージを返します。
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ofkt::platform::SystemTray;
    ///
    /// let mut tray = SystemTray::new();
    /// tray.build().expect("トレイアイコンの構築に失敗しました");
    /// ```
    pub fn build(&mut self) -> Result<(), String> {
        // メニューアイテム作成
        let open_item = MenuItem::new("開く", true, None);
        let settings_item = MenuItem::new("設定", true, None);
        let exit_item = MenuItem::new("終了", true, None);

        // メニュー作成
        let menu = Menu::new();
        menu.append(&open_item)
            .map_err(|e| format!("メニュー追加失敗: {}", e))?;
        menu.append(&settings_item)
            .map_err(|e| format!("メニュー追加失敗: {}", e))?;
        menu.append(&exit_item)
            .map_err(|e| format!("メニュー追加失敗: {}", e))?;

        // アイコン作成
        let icon = self.load_icon()?;

        // トレイアイコン作成
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_icon(icon)
            .with_tooltip("Ofkt")
            .build()
            .map_err(|e| format!("トレイアイコン作成失敗: {}", e))?;

        // IDを保存（MenuIdの内部Stringにアクセス）
        self.menu_item_open_id = Some(open_item.id().0.clone());
        self.menu_item_settings_id = Some(settings_item.id().0.clone());
        self.menu_item_exit_id = Some(exit_item.id().0.clone());

        self.tray_icon = Some(tray_icon);
        self.menu = Some(menu);

        Ok(())
    }

    /// アイコンを読み込み
    ///
    /// resources/icon.png を読み込みます。
    /// ファイルが見つからない場合はデフォルトアイコンを使用します。
    fn load_icon(&self) -> Result<Icon, String> {
        // resources/icon.png を試みる
        let icon_path = std::path::Path::new("resources/icon.png");

        if icon_path.exists() {
            let image = image::open(icon_path)
                .map_err(|e| format!("アイコン画像の読み込み失敗: {}", e))?;

            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();

            Icon::from_rgba(rgba.into_raw(), width, height)
                .map_err(|e| format!("アイコンの作成失敗: {}", e))
        } else {
            // デフォルトアイコン（32x32の単色アイコン）を作成
            let size = 32;
            let mut rgba = vec![0u8; (size * size * 4) as usize];

            // 青色の円を描画
            for y in 0..size {
                for x in 0..size {
                    let dx = x as f32 - size as f32 / 2.0;
                    let dy = y as f32 - size as f32 / 2.0;
                    let dist = (dx * dx + dy * dy).sqrt();

                    let idx = ((y * size + x) * 4) as usize;

                    if dist < size as f32 / 2.0 - 2.0 {
                        rgba[idx] = 0;       // R
                        rgba[idx + 1] = 120; // G
                        rgba[idx + 2] = 215; // B
                        rgba[idx + 3] = 255; // A
                    } else {
                        rgba[idx + 3] = 0;   // 透明
                    }
                }
            }

            Icon::from_rgba(rgba, size, size)
                .map_err(|e| format!("デフォルトアイコンの作成失敗: {}", e))
        }
    }

    /// イベントを処理
    ///
    /// トレイメニューからのイベントをポーリングし、
    /// 対応する `TrayEvent` を返します。
    ///
    /// # Returns
    ///
    /// イベントがある場合は `Some(TrayEvent)`、ない場合は `None` を返します。
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ofkt::platform::{SystemTray, TrayEvent};
    ///
    /// let mut tray = SystemTray::new();
    /// tray.build().unwrap();
    ///
    /// loop {
    ///     if let Some(event) = tray.handle_events() {
    ///         match event {
    ///             TrayEvent::Open => println!("開くが選択されました"),
    ///             TrayEvent::Settings => println!("設定が選択されました"),
    ///             TrayEvent::Exit => break,
    ///         }
    ///     }
    /// }
    /// ```
    pub fn handle_events(&self) -> Option<TrayEvent> {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            let event_id = &event.id.0;

            if Some(event_id) == self.menu_item_open_id.as_ref() {
                return Some(TrayEvent::Open);
            } else if Some(event_id) == self.menu_item_settings_id.as_ref() {
                return Some(TrayEvent::Settings);
            } else if Some(event_id) == self.menu_item_exit_id.as_ref() {
                return Some(TrayEvent::Exit);
            }
        }

        None
    }

    /// アクティブ状態に設定
    ///
    /// 将来的にアクティブ時のアイコンに切り替えます。
    /// 現在は何もしません。
    ///
    /// # Returns
    ///
    /// 成功時は `Ok(())`、失敗時はエラーメッセージを返します。
    pub fn set_active(&mut self) -> Result<(), String> {
        // 将来のアイコン切り替え用
        // 現時点では何もしない
        Ok(())
    }

    /// 非アクティブ状態に設定
    ///
    /// 将来的に非アクティブ時のアイコンに切り替えます。
    /// 現在は何もしません。
    ///
    /// # Returns
    ///
    /// 成功時は `Ok(())`、失敗時はエラーメッセージを返します。
    pub fn set_inactive(&mut self) -> Result<(), String> {
        // 将来のアイコン切り替え用
        // 現時点では何もしない
        Ok(())
    }
}

impl Default for SystemTray {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let tray = SystemTray::new();
        assert!(tray.tray_icon.is_none());
        assert!(tray.menu.is_none());
        assert!(tray.menu_item_open_id.is_none());
        assert!(tray.menu_item_settings_id.is_none());
        assert!(tray.menu_item_exit_id.is_none());
    }

    #[test]
    fn test_default() {
        let tray = SystemTray::default();
        assert!(tray.tray_icon.is_none());
        assert!(tray.menu.is_none());
    }

    #[test]
    fn test_tray_event_enum() {
        // TrayEvent の基本動作をテスト
        let open = TrayEvent::Open;
        let settings = TrayEvent::Settings;
        let exit = TrayEvent::Exit;

        // PartialEq のテスト
        assert_eq!(open, TrayEvent::Open);
        assert_eq!(settings, TrayEvent::Settings);
        assert_eq!(exit, TrayEvent::Exit);

        assert_ne!(open, settings);
        assert_ne!(settings, exit);
        assert_ne!(exit, open);

        // Clone のテスト
        let open_clone = open.clone();
        assert_eq!(open, open_clone);

        // Debug のテスト（パニックしないことを確認）
        let debug_str = format!("{:?}", open);
        assert!(debug_str.contains("Open"));
    }

    #[test]
    fn test_load_icon() {
        let tray = SystemTray::new();

        // デフォルトアイコンが作成されることを確認
        let icon_result = tray.load_icon();
        assert!(icon_result.is_ok(), "デフォルトアイコンの作成に失敗しました");
    }

    #[test]
    fn test_set_active_inactive() {
        let mut tray = SystemTray::new();

        // 現時点では何もしないが、エラーにならないことを確認
        assert!(tray.set_active().is_ok());
        assert!(tray.set_inactive().is_ok());
    }

    #[test]
    fn test_handle_events_without_build() {
        let tray = SystemTray::new();

        // build() を呼ばない状態でも handle_events() は動作する
        // （イベントがないので None が返る）
        assert_eq!(tray.handle_events(), None);
    }
}
