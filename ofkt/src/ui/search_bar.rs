use eframe::egui;
use std::time::{Duration, Instant};

/// 検索バーのイベント
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchBarEvent {
    /// 検索クエリが変更された
    pub changed: bool,
    /// 検索バーがクリアされた（Escapeキー or クリアボタン）
    pub cleared: bool,
    /// 検索が確定された（Enterキー）
    pub submitted: bool,
    /// 検索バーがフォーカスを持っているか
    pub has_focus: bool,
}

impl Default for SearchBarEvent {
    fn default() -> Self {
        Self {
            changed: false,
            cleared: false,
            submitted: false,
            has_focus: false,
        }
    }
}

/// 検索バーコンポーネント
pub struct SearchBar {
    /// プレースホルダーテキスト
    placeholder: String,
    /// 検索バーのID（フォーカス制御用）
    id: egui::Id,
}

impl Default for SearchBar {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchBar {
    /// 新しい SearchBar を作成
    pub fn new() -> Self {
        Self {
            placeholder: "検索...".to_string(),
            id: egui::Id::new("search_bar"),
        }
    }

    /// プレースホルダーを設定
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// 検索バーにフォーカスを要求
    pub fn request_focus(&self, ctx: &egui::Context) {
        ctx.memory_mut(|mem| mem.request_focus(self.id));
    }

    /// 検索バーを描画
    ///
    /// # 戻り値
    /// SearchBarEvent - 検索バーで発生したイベント情報
    pub fn render(&self, ui: &mut egui::Ui, query: &mut String) -> SearchBarEvent {
        let mut event = SearchBarEvent::default();

        let text_edit_response = ui.horizontal(|ui| {
            // 検索アイコン
            ui.label("🔍");

            // 検索入力フィールド
            let response = ui.add(
                egui::TextEdit::singleline(query)
                    .id(self.id)
                    .hint_text(&self.placeholder)
                    .desired_width(ui.available_width() - 30.0)
            );

            if response.changed() {
                event.changed = true;
            }

            // クリアボタン（検索クエリが空でない場合のみ表示）
            if !query.is_empty() {
                if ui.button("✖").clicked() {
                    query.clear();
                    event.changed = true;
                    event.cleared = true;
                }
            }

            response
        }).inner;

        // フォーカス状態を記録
        event.has_focus = text_edit_response.has_focus();

        // Escapeキーで検索クリア
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            if !query.is_empty() {
                query.clear();
                event.changed = true;
                event.cleared = true;
            }
        }

        // Enterキーで検索確定
        if text_edit_response.lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter))
        {
            event.submitted = true;
        }

        event
    }
}

/// 検索デバウンサー
pub struct SearchDebouncer {
    last_query: String,
    last_update: Instant,
    debounce_duration: Duration,
}

impl Default for SearchDebouncer {
    fn default() -> Self {
        Self::new(Duration::from_millis(150))
    }
}

impl SearchDebouncer {
    /// 新しい SearchDebouncer を作成
    pub fn new(debounce_duration: Duration) -> Self {
        Self {
            last_query: String::new(),
            last_update: Instant::now(),
            debounce_duration,
        }
    }

    /// 検索を実行すべきかチェック
    ///
    /// # 引数
    /// * `current_query` - 現在の検索クエリ
    ///
    /// # 戻り値
    /// 検索を実行すべき場合は true を返す
    pub fn should_search(&mut self, current_query: &str) -> bool {
        let now = Instant::now();
        let query_changed = self.last_query != current_query;
        let debounce_elapsed = now.duration_since(self.last_update) >= self.debounce_duration;

        if query_changed {
            self.last_update = now;
            self.last_query = current_query.to_string();

            // クエリが変更されてデバウンス期間が経過している場合のみ検索
            debounce_elapsed
        } else {
            false
        }
    }

    /// 即座に検索を実行すべきかチェック（Enter キー押下時など）
    pub fn force_search(&mut self) {
        self.last_update = Instant::now() - self.debounce_duration;
    }
}
