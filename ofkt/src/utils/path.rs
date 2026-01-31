use std::path::{Path, PathBuf};

/// パスを正規化する（OS対応）
///
/// Windowsでは大文字小文字を区別せず、シンボリックリンクを解決する
pub fn normalize_path(path: &Path) -> Result<PathBuf, std::io::Error> {
    path.canonicalize()
}

/// 2つのパスが同一か比較する（OS対応）
///
/// Windowsでは大文字小文字を区別しない
#[cfg(target_os = "windows")]
pub fn paths_equal(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a_canon), Ok(b_canon)) => a_canon == b_canon,
        _ => false,
    }
}

#[cfg(not(target_os = "windows"))]
pub fn paths_equal(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a_canon), Ok(b_canon)) => a_canon == b_canon,
        _ => a == b, // フォールバック
    }
}

/// パスのリストを正規化する
pub fn normalize_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.iter()
        .filter_map(|p| normalize_path(p).ok())
        .collect()
}
