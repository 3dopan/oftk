use eframe::egui;
use crate::data::models::Config;

/// 設定画面でのアクション
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsAction {
    /// 設定を保存
    Save,
    /// キャンセル
    Cancel,
}

/// 設定画面コンポーネント
pub struct Settings {
    /// 現在の設定
    config: Config,
    /// ホットキー入力用の一時文字列
    temp_hotkey: String,
}

impl Settings {
    /// 新しい Settings を作成
    ///
    /// # 引数
    /// * `config` - 現在の設定
    pub fn new(config: Config) -> Self {
        // ホットキー設定から初期値を構築
        let temp_hotkey = Self::build_hotkey_string(&config);

        Self {
            config,
            temp_hotkey,
        }
    }

    /// ホットキー文字列を構築
    fn build_hotkey_string(config: &Config) -> String {
        let mut parts = config.hotkey.modifiers.clone();
        parts.push(config.hotkey.key.clone());
        parts.join("+")
    }

    /// 設定画面を描画
    ///
    /// # 引数
    /// * `ui` - egui UI コンテキスト
    ///
    /// # 戻り値
    /// ユーザーがアクションを実行した場合は Some(SettingsAction) を返す
    pub fn render(&mut self, ui: &mut egui::Ui) -> Option<SettingsAction> {
        let mut action = None;

        ui.heading("設定");
        ui.separator();

        // ホットキー設定セクション
        ui.group(|ui| {
            ui.label("ホットキー設定");
            ui.horizontal(|ui| {
                ui.label("キー組み合わせ:");
                ui.text_edit_singleline(&mut self.temp_hotkey);
            });
            ui.checkbox(&mut self.config.hotkey.enabled, "ホットキーを有効化");
            ui.label("例: Ctrl+Shift+O");
        });

        ui.add_space(10.0);

        // ウィンドウ設定セクション
        ui.group(|ui| {
            ui.label("ウィンドウ設定");

            ui.horizontal(|ui| {
                ui.label("幅:");
                ui.add(egui::Slider::new(&mut self.config.window.width, 200.0..=1000.0));
            });

            ui.horizontal(|ui| {
                ui.label("高さ:");
                ui.add(egui::Slider::new(&mut self.config.window.height, 400.0..=2000.0));
            });

            ui.checkbox(&mut self.config.window.always_on_top, "常に最前面に表示");
            ui.checkbox(&mut self.config.window.decorations, "ウィンドウ装飾を表示");
        });

        ui.add_space(10.0);

        // テーマ設定セクション
        ui.group(|ui| {
            ui.label("テーマ設定");
            ui.radio_value(&mut self.config.theme.mode, "system".to_string(), "システム設定に従う");
            ui.radio_value(&mut self.config.theme.mode, "light".to_string(), "ライトモード");
            ui.radio_value(&mut self.config.theme.mode, "dark".to_string(), "ダークモード");
        });

        ui.add_space(10.0);

        // 画面端トリガー設定セクション
        ui.group(|ui| {
            ui.label("画面端トリガー設定");
            ui.checkbox(&mut self.config.edge_trigger.enabled, "画面端トリガーを有効化");

            if self.config.edge_trigger.enabled {
                ui.horizontal(|ui| {
                    ui.label("トリガー位置:");
                    ui.radio_value(&mut self.config.edge_trigger.edge, "left".to_string(), "左");
                    ui.radio_value(&mut self.config.edge_trigger.edge, "right".to_string(), "右");
                    ui.radio_value(&mut self.config.edge_trigger.edge, "top".to_string(), "上");
                    ui.radio_value(&mut self.config.edge_trigger.edge, "bottom".to_string(), "下");
                });

                ui.horizontal(|ui| {
                    ui.label("遅延(ms):");
                    ui.add(egui::Slider::new(&mut self.config.edge_trigger.delay_ms, 0..=1000));
                });

                ui.horizontal(|ui| {
                    ui.label("トリガー幅(px):");
                    ui.add(egui::Slider::new(&mut self.config.edge_trigger.trigger_width, 1..=50));
                });
            }
        });

        ui.add_space(10.0);

        // 自動起動設定セクション
        ui.group(|ui| {
            ui.label("自動起動設定");
            ui.checkbox(&mut self.config.autostart.enabled, "Windows起動時に自動起動");
        });

        ui.add_space(10.0);

        // 検索設定セクション
        ui.group(|ui| {
            ui.label("検索設定");
            ui.checkbox(&mut self.config.search.incremental, "インクリメンタル検索");
            ui.checkbox(&mut self.config.search.fuzzy_match, "あいまい検索");
            ui.checkbox(&mut self.config.search.search_paths, "パスを検索対象に含める");
            ui.checkbox(&mut self.config.search.search_aliases, "エイリアスを検索対象に含める");
            ui.checkbox(&mut self.config.search.case_sensitive, "大文字小文字を区別");
        });

        ui.add_space(10.0);

        // ファイル操作設定セクション
        ui.group(|ui| {
            ui.label("ファイル操作設定");
            ui.checkbox(&mut self.config.file_operations.confirm_delete, "削除前に確認");
            ui.checkbox(&mut self.config.file_operations.use_trash, "ゴミ箱に移動");

            ui.horizontal(|ui| {
                ui.label("デフォルト開き方:");
                ui.radio_value(&mut self.config.file_operations.default_open_action, "open".to_string(), "開く");
                ui.radio_value(&mut self.config.file_operations.default_open_action, "explore".to_string(), "エクスプローラーで開く");
                ui.radio_value(&mut self.config.file_operations.default_open_action, "copy_path".to_string(), "パスをコピー");
            });
        });

        ui.add_space(20.0);

        // 保存/キャンセルボタン
        ui.horizontal(|ui| {
            if ui.button("保存").clicked() {
                // ホットキー文字列を解析して設定に反映
                self.parse_hotkey_string();
                action = Some(SettingsAction::Save);
            }
            if ui.button("キャンセル").clicked() {
                action = Some(SettingsAction::Cancel);
            }
        });

        action
    }

    /// ホットキー文字列を解析して設定に反映
    fn parse_hotkey_string(&mut self) {
        let parts: Vec<String> = self.temp_hotkey
            .split('+')
            .map(|s| s.trim().to_string())
            .collect();

        if !parts.is_empty() {
            // 最後の要素をキーとして、それ以外を修飾キーとする
            let key = parts.last().unwrap().clone();
            let modifiers: Vec<String> = parts[..parts.len().saturating_sub(1)].to_vec();

            self.config.hotkey.key = key;
            self.config.hotkey.modifiers = modifiers;
        }
    }

    /// 現在の設定を取得
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    /// 設定を更新
    pub fn update_config(&mut self, config: Config) {
        self.temp_hotkey = Self::build_hotkey_string(&config);
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::models::*;

    fn create_test_config() -> Config {
        Config {
            version: "1.0.0".to_string(),
            window: WindowConfig {
                width: 600.0,
                height: 800.0,
                position: WindowPosition { x: 0.0, y: 0.0 },
                always_on_top: true,
                decorations: true,
            },
            hotkey: HotkeyConfig {
                enabled: true,
                modifiers: vec!["Ctrl".to_string(), "Shift".to_string()],
                key: "O".to_string(),
            },
            edge_trigger: EdgeTriggerConfig {
                enabled: false,
                edge: "right".to_string(),
                delay_ms: 300,
                trigger_width: 5,
            },
            autostart: AutostartConfig {
                enabled: false,
            },
            theme: ThemeConfig {
                mode: "system".to_string(),
                custom_accent_color: None,
            },
            search: SearchConfig {
                incremental: true,
                fuzzy_match: true,
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
    fn test_settings_new() {
        let config = create_test_config();
        let settings = Settings::new(config.clone());

        assert_eq!(settings.get_config().version, "1.0.0");
        assert_eq!(settings.temp_hotkey, "Ctrl+Shift+O");
    }

    #[test]
    fn test_build_hotkey_string() {
        let config = create_test_config();
        let hotkey_string = Settings::build_hotkey_string(&config);

        assert_eq!(hotkey_string, "Ctrl+Shift+O");
    }

    #[test]
    fn test_parse_hotkey_string() {
        let config = create_test_config();
        let mut settings = Settings::new(config);

        settings.temp_hotkey = "Alt+F4".to_string();
        settings.parse_hotkey_string();

        assert_eq!(settings.config.hotkey.modifiers, vec!["Alt".to_string()]);
        assert_eq!(settings.config.hotkey.key, "F4");
    }

    #[test]
    fn test_update_config() {
        let config = create_test_config();
        let mut settings = Settings::new(config.clone());

        let mut new_config = config.clone();
        new_config.window.width = 800.0;
        new_config.hotkey.key = "P".to_string();

        settings.update_config(new_config);

        assert_eq!(settings.config.window.width, 800.0);
        assert_eq!(settings.temp_hotkey, "Ctrl+Shift+P");
    }

    #[test]
    fn test_settings_action_enum() {
        let save_action = SettingsAction::Save;
        let cancel_action = SettingsAction::Cancel;

        assert_ne!(save_action, cancel_action);
        assert_eq!(save_action, SettingsAction::Save);
    }
}
