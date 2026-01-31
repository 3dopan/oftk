mod data;
mod utils;
mod app;
mod ui;
mod platform;
mod core;

use anyhow::Result;
use log::info;

fn main() -> Result<()> {
    // ロガー初期化
    utils::logger::init_logger()?;

    info!("Ofkt 起動中...");

    // eframe の NativeOptions を設定
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([300.0, 1080.0])  // 幅300px、高さ画面全体
            .with_position([1620.0, 0.0])      // 初期位置
            .with_resizable(true)               // リサイズ可能
            .with_decorations(true)             // ウィンドウ装飾あり
            .with_transparent(false),           // 透明度なし
        persistence_path: Some(
            dirs::config_dir()
                .unwrap()
                .join("ofkt")
                .join("window_state.json")
        ),
        ..Default::default()
    };

    info!("eframe 起動");

    // eframe を起動
    eframe::run_native(
        "Ofkt - ファイル管理ツール",
        native_options,
        Box::new(|cc| {
            // Windowsシステムフォントを読み込む
            use eframe::egui::{FontDefinitions, FontData, FontFamily};
            use std::sync::Arc;

            let mut fonts = FontDefinitions::default();

            // 日本語フォントを読み込む（優先順に試行）
            let font_paths = vec![
                r"C:\Windows\Fonts\YuGothR.ttc",    // Yu Gothic UI Regular
                r"C:\Windows\Fonts\meiryo.ttc",     // メイリオ
                r"C:\Windows\Fonts\msgothic.ttc",   // MS Gothic
            ];

            for font_path in font_paths {
                if let Ok(font_bytes) = std::fs::read(font_path) {
                    info!("フォント読み込み成功: {}", font_path);

                    fonts.font_data.insert(
                        "japanese".to_owned(),
                        FontData::from_owned(font_bytes).into()
                    );

                    // Proportionalフォントファミリーの先頭に追加
                    fonts.families
                        .entry(FontFamily::Proportional)
                        .or_default()
                        .insert(0, "japanese".to_owned());

                    // Monospaceフォントファミリーの先頭に追加
                    fonts.families
                        .entry(FontFamily::Monospace)
                        .or_default()
                        .insert(0, "japanese".to_owned());

                    // フォント設定を適用
                    cc.egui_ctx.set_fonts(fonts);

                    info!("日本語フォント設定完了");
                    break;
                } else {
                    log::warn!("フォントファイルが見つかりません: {}", font_path);
                }
            }

            Ok(Box::new(app::OfktApp::new()))
        }),
    ).map_err(|e| anyhow::anyhow!("eframe 起動エラー: {}", e))?;

    info!("Ofkt 終了");

    Ok(())
}
