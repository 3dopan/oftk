use std::path::PathBuf;
use crate::utils::path::normalize_paths;

/// クリップボードの操作モード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardMode {
    /// コピーモード
    Copy,
    /// 切り取りモード（移動）
    Cut,
}

/// クリップボード管理
#[derive(Debug, Clone)]
pub struct ClipboardState {
    /// クリップボードに保持されているパスのリスト
    pub paths: Vec<PathBuf>,

    /// クリップボードモード（コピーor切り取り）
    pub mode: ClipboardMode,

    /// クリップボードが有効かどうか
    pub is_active: bool,
}

impl ClipboardState {
    /// 新しいクリップボード状態を作成
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            mode: ClipboardMode::Copy,
            is_active: false,
        }
    }

    /// クリップボードにパスをコピー
    pub fn copy(&mut self, paths: Vec<PathBuf>) {
        log::info!("ClipboardState::copy() called");

        // 変更前の状態をログ出力
        log::debug!(
            "Before: paths={}, mode={:?}, is_active={}",
            self.paths.len(),
            self.mode,
            self.is_active
        );

        // 受け取ったパスの情報をログ出力
        log::info!("Received {} paths to copy", paths.len());
        for (i, path) in paths.iter().enumerate() {
            log::debug!("  Path[{}]: {}", i, path.display());
        }

        // パスを正規化して保存
        self.paths = normalize_paths(paths);
        self.mode = ClipboardMode::Copy;
        self.is_active = true;

        // 変更後の状態をログ出力
        log::debug!(
            "After: paths={}, mode={:?}, is_active={}",
            self.paths.len(),
            self.mode,
            self.is_active
        );
    }

    /// クリップボードにパスを切り取り
    pub fn cut(&mut self, paths: Vec<PathBuf>) {
        log::info!("ClipboardState::cut() called");

        // 変更前の状態をログ出力
        log::debug!(
            "Before: paths={}, mode={:?}, is_active={}",
            self.paths.len(),
            self.mode,
            self.is_active
        );

        // 受け取ったパスの情報をログ出力
        log::info!("Received {} paths to cut", paths.len());
        for (i, path) in paths.iter().enumerate() {
            log::debug!("  Path[{}]: {}", i, path.display());
        }

        // パスを正規化して保存
        self.paths = normalize_paths(paths);
        self.mode = ClipboardMode::Cut;
        self.is_active = true;

        // 変更後の状態をログ出力
        log::debug!(
            "After: paths={}, mode={:?}, is_active={}",
            self.paths.len(),
            self.mode,
            self.is_active
        );
    }

    /// クリップボードをクリア
    pub fn clear(&mut self) {
        log::info!("ClipboardState::clear() called");

        // クリア前の状態をログ出力
        log::debug!(
            "Clearing clipboard: had {} paths in {:?} mode",
            self.paths.len(),
            self.mode
        );

        // クリップボードをクリア
        self.paths.clear();
        self.is_active = false;

        // クリア後の確認ログ
        log::debug!("Clipboard cleared: is_empty={}", self.is_empty());
    }

    /// クリップボードが空かどうか
    pub fn is_empty(&self) -> bool {
        !self.is_active || self.paths.is_empty()
    }
}

impl Default for ClipboardState {
    fn default() -> Self {
        Self::new()
    }
}

/// コピー時のファイル名を生成（同一ディレクトリの場合）
pub fn generate_copy_name(original_path: &std::path::Path, dest_dir: &std::path::Path) -> PathBuf {
    let file_name = original_path.file_stem().unwrap_or_default();
    let extension = original_path.extension();

    let mut counter = 0;
    loop {
        let new_name = if counter == 0 {
            format!("{} (コピー)", file_name.to_string_lossy())
        } else {
            format!("{} (コピー {})", file_name.to_string_lossy(), counter + 1)
        };

        let new_path = if let Some(ext) = extension {
            dest_dir.join(format!("{}.{}", new_name, ext.to_string_lossy()))
        } else {
            dest_dir.join(new_name)
        };

        if !new_path.exists() {
            return new_path;
        }

        counter += 1;

        // 無限ループ防止
        if counter > 9999 {
            return dest_dir.join(format!("{}_{}", file_name.to_string_lossy(), uuid::Uuid::new_v4()));
        }
    }
}
