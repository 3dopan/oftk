use std::env;

#[cfg(target_os = "windows")]
use windows::core::HSTRING;
#[cfg(target_os = "windows")]
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};

const APP_NAME: &str = "Ofkt";
const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// 自動起動設定を管理する構造体
pub struct AutostartManager;

impl AutostartManager {
    /// 新しい AutostartManager インスタンスを作成
    pub fn new() -> Self {
        Self
    }

    /// 自動起動を有効化（実行ファイルパスをレジストリに登録）
    ///
    /// # Returns
    /// - `Ok(())`: 自動起動の有効化に成功
    /// - `Err(String)`: エラーメッセージ
    pub fn enable(&self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            // 現在の実行ファイルパスを取得
            let exe_path = env::current_exe()
                .map_err(|e| format!("実行ファイルパス取得失敗: {}", e))?;

            let exe_path_str = exe_path
                .to_str()
                .ok_or_else(|| "パスの変換に失敗しました".to_string())?;

            unsafe {
                // レジストリキーを開く
                let key_path = HSTRING::from(RUN_KEY_PATH);
                let mut key = Default::default();
                RegOpenKeyExW(HKEY_CURRENT_USER, &key_path, 0, KEY_WRITE, &mut key)
                    .ok()
                    .map_err(|e| format!("レジストリキーを開けません: {}", e))?;

                // レジストリに書き込み
                let app_name = HSTRING::from(APP_NAME);
                let value = HSTRING::from(exe_path_str);
                let value_bytes = value.as_wide();

                let result = RegSetValueExW(
                    key,
                    &app_name,
                    0,
                    REG_SZ,
                    Some(std::slice::from_raw_parts(
                        value_bytes.as_ptr() as *const u8,
                        value_bytes.len() * 2,
                    )),
                );

                RegCloseKey(key).ok();

                result.ok().map_err(|e| format!("レジストリ書き込み失敗: {}", e))?;
            }

            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err("自動起動はWindowsでのみサポートされています".to_string())
        }
    }

    /// 自動起動を無効化（レジストリキーを削除）
    ///
    /// # Returns
    /// - `Ok(())`: 自動起動の無効化に成功
    /// - `Err(String)`: エラーメッセージ
    pub fn disable(&self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                let key_path = HSTRING::from(RUN_KEY_PATH);
                let mut key = Default::default();
                RegOpenKeyExW(HKEY_CURRENT_USER, &key_path, 0, KEY_WRITE, &mut key)
                    .ok()
                    .map_err(|e| format!("レジストリキーを開けません: {}", e))?;

                let app_name = HSTRING::from(APP_NAME);
                let result = RegDeleteValueW(key, &app_name);

                RegCloseKey(key).ok();

                result.ok().map_err(|e| format!("レジストリキー削除失敗: {}", e))?;
            }

            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err("自動起動はWindowsでのみサポートされています".to_string())
        }
    }

    /// 自動起動が有効かどうかを確認
    ///
    /// # Returns
    /// - `true`: 自動起動が有効
    /// - `false`: 自動起動が無効またはWindows以外のOS
    pub fn is_enabled(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                let key_path = HSTRING::from(RUN_KEY_PATH);
                let mut key = Default::default();
                if RegOpenKeyExW(HKEY_CURRENT_USER, &key_path, 0, KEY_READ, &mut key).is_err() {
                    return false;
                }

                let app_name = HSTRING::from(APP_NAME);
                let mut buffer = vec![0u8; 1024];
                let mut buffer_size = buffer.len() as u32;

                let result = RegQueryValueExW(
                    key,
                    &app_name,
                    None,
                    None,
                    Some(buffer.as_mut_ptr()),
                    Some(&mut buffer_size),
                );

                RegCloseKey(key).ok();

                result.is_ok()
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }
}

impl Default for AutostartManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autostart_manager_creation() {
        let manager = AutostartManager::new();
        // 作成できることを確認
        assert_eq!(std::mem::size_of_val(&manager), 0);
    }

    #[test]
    fn test_autostart_manager_default() {
        let manager = AutostartManager::default();
        // デフォルト作成できることを確認
        assert_eq!(std::mem::size_of_val(&manager), 0);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_enable_disable_cycle() {
        let manager = AutostartManager::new();

        // まず無効化（既存の設定をクリア）
        let _ = manager.disable();
        assert!(!manager.is_enabled());

        // 有効化
        match manager.enable() {
            Ok(_) => {
                assert!(manager.is_enabled());

                // 無効化
                match manager.disable() {
                    Ok(_) => assert!(!manager.is_enabled()),
                    Err(e) => eprintln!("無効化失敗: {}", e),
                }
            }
            Err(e) => eprintln!("有効化失敗: {}", e),
        }
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_non_windows_behavior() {
        let manager = AutostartManager::new();

        // Windows以外ではis_enabledはfalseを返す
        assert!(!manager.is_enabled());

        // enable/disableはエラーを返す
        assert!(manager.enable().is_err());
        assert!(manager.disable().is_err());
    }
}
