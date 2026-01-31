use anyhow::Result;
use log::{LevelFilter, Log, Metadata, Record};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// マルチターゲットロガー（標準出力 + ファイル）
struct MultiLogger {
    log_file: Mutex<File>,
    level: LevelFilter,
}

impl MultiLogger {
    fn new(log_file_path: PathBuf, level: LevelFilter) -> Result<Self> {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)?;

        Ok(Self {
            log_file: Mutex::new(log_file),
            level,
        })
    }

    fn format_log(&self, record: &Record) -> String {
        format!(
            "[{} {} {}] {}\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.target(),
            record.args()
        )
    }
}

impl Log for MultiLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let formatted = self.format_log(record);

            // 標準出力に出力
            print!("{}", formatted);

            // ファイルに出力
            if let Ok(mut file) = self.log_file.lock() {
                let _ = file.write_all(formatted.as_bytes());
                let _ = file.flush();
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.log_file.lock() {
            let _ = file.flush();
        }
    }
}

/// ロガーを初期化
pub fn init_logger() -> Result<()> {
    // ログディレクトリを作成
    let log_dir = ensure_log_dir()?;
    let log_file_path = log_dir.join("ofkt.log");

    // ログレベル設定（環境変数 RUST_LOG があればそれを使用、なければ Debug）
    let level = if let Ok(rust_log) = std::env::var("RUST_LOG") {
        match rust_log.to_lowercase().as_str() {
            "trace" => LevelFilter::Trace,
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            _ => LevelFilter::Debug,
        }
    } else {
        LevelFilter::Debug // デフォルトはDebug
    };

    // マルチロガーを作成
    let logger = MultiLogger::new(log_file_path, level)?;

    // ロガーを設定
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(level);

    Ok(())
}

/// ログディレクトリを作成
pub fn ensure_log_dir() -> Result<std::path::PathBuf> {
    let config_dir = crate::data::storage::get_config_dir()?;
    let log_dir = config_dir.join("logs");

    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)?;
    }

    Ok(log_dir)
}
