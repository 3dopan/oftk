use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DriveInfo {
    pub name: String,
    pub path: PathBuf,
    pub drive_type: DriveType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DriveType {
    Fixed,      // ローカルディスク（C:, D:など）
    Removable,  // USBドライブ
    Network,    // ネットワークドライブ
    WSL,        // WSL
    QuickAccess, // クイックアクセス
}

/// Windowsのドライブ一覧を取得
pub fn get_drives() -> Vec<DriveInfo> {
    use windows::Win32::Storage::FileSystem::GetDriveTypeW;
    use windows::core::PCWSTR;

    let mut drives = Vec::new();

    // A-Z のドライブをチェック
    for letter in b'A'..=b'Z' {
        let drive_path = format!("{}:\\", letter as char);
        let drive_wide: Vec<u16> = drive_path.encode_utf16().chain(Some(0)).collect();

        unsafe {
            let drive_type = GetDriveTypeW(PCWSTR(drive_wide.as_ptr()));

            if drive_type != 1 { // 1 = DRIVE_NO_ROOT_DIR
                let dtype = match drive_type {
                    3 => DriveType::Fixed,      // DRIVE_FIXED
                    2 => DriveType::Removable,  // DRIVE_REMOVABLE
                    4 => DriveType::Network,    // DRIVE_REMOTE
                    _ => DriveType::Fixed,
                };

                drives.push(DriveInfo {
                    name: format!("{} ドライブ", letter as char),
                    path: PathBuf::from(&drive_path),
                    drive_type: dtype,
                });
            }
        }
    }

    drives
}

/// WSLディストリビューション一覧を取得
pub fn get_wsl_distributions() -> Vec<DriveInfo> {
    let wsl_root = PathBuf::from(r"\\wsl$");
    let mut wsl_drives = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&wsl_root) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Some(name) = entry.file_name().to_str() {
                wsl_drives.push(DriveInfo {
                    name: format!("WSL: {}", name),
                    path: entry.path(),
                    drive_type: DriveType::WSL,
                });
            }
        }
    }

    wsl_drives
}

/// クイックアクセスパスを取得
pub fn get_quick_access() -> Vec<DriveInfo> {
    let mut quick = Vec::new();

    if let Some(home) = dirs::home_dir() {
        quick.push(DriveInfo {
            name: "ホーム".to_string(),
            path: home.clone(),
            drive_type: DriveType::QuickAccess,
        });
    }

    if let Some(desktop) = dirs::desktop_dir() {
        quick.push(DriveInfo {
            name: "デスクトップ".to_string(),
            path: desktop,
            drive_type: DriveType::QuickAccess,
        });
    }

    if let Some(docs) = dirs::document_dir() {
        quick.push(DriveInfo {
            name: "ドキュメント".to_string(),
            path: docs,
            drive_type: DriveType::QuickAccess,
        });
    }

    if let Some(downloads) = dirs::download_dir() {
        quick.push(DriveInfo {
            name: "ダウンロード".to_string(),
            path: downloads,
            drive_type: DriveType::QuickAccess,
        });
    }

    quick
}
