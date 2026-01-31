use crate::ui::theme::Theme;

/// システムテーマを検出
pub fn detect_system_theme() -> Theme {
    // Windows の場合のみレジストリから取得
    #[cfg(target_os = "windows")]
    {
        match read_windows_theme() {
            Ok(theme) => theme,
            Err(e) => {
                log::warn!("システムテーマの検出に失敗、ダークモードを使用: {}", e);
                Theme::Dark
            }
        }
    }

    // Windows 以外はダークモードをデフォルトとする
    #[cfg(not(target_os = "windows"))]
    {
        Theme::Dark
    }
}

#[cfg(target_os = "windows")]
fn read_windows_theme() -> anyhow::Result<Theme> {
    use windows::Win32::System::Registry::*;
    use windows::core::HSTRING;

    unsafe {
        // レジストリキーを開く
        // HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize
        let subkey = HSTRING::from("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
        let mut key = HKEY::default();

        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            &subkey,
            0,
            KEY_READ,
            &mut key,
        );

        if result.is_err() {
            return Err(anyhow::anyhow!("レジストリキーを開けませんでした"));
        }

        // AppsUseLightTheme の値を読み取り
        let value_name = HSTRING::from("AppsUseLightTheme");
        let mut data: u32 = 0;
        let mut data_size: u32 = std::mem::size_of::<u32>() as u32;

        let result = RegQueryValueExW(
            key,
            &value_name,
            None,
            None,
            Some(&mut data as *mut u32 as *mut u8),
            Some(&mut data_size),
        );

        // レジストリキーを閉じる
        let _ = RegCloseKey(key);

        if result.is_err() {
            return Err(anyhow::anyhow!("レジストリ値の読み取りに失敗しました"));
        }

        // 0 = Dark, 1 = Light
        if data == 0 {
            Ok(Theme::Dark)
        } else {
            Ok(Theme::Light)
        }
    }
}
