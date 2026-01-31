use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};

/// グローバルホットキーを管理する構造体
pub struct HotkeyManager {
    manager: GlobalHotKeyManager,
    hotkey: Option<HotKey>,
}

impl HotkeyManager {
    /// 新しいHotkeyManagerを作成する
    ///
    /// # エラー
    /// GlobalHotKeyManagerの作成に失敗した場合、エラーメッセージを返す
    pub fn new() -> Result<Self, String> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| format!("ホットキーマネージャー作成失敗: {}", e))?;
        Ok(Self {
            manager,
            hotkey: None,
        })
    }

    /// ホットキーを登録する
    ///
    /// # 引数
    /// * `modifiers` - 修飾キー（Ctrl、Shift、Altなど）
    /// * `code` - キーコード（O、Aなど）
    ///
    /// # エラー
    /// ホットキーの登録に失敗した場合、エラーメッセージを返す
    pub fn register(&mut self, modifiers: Modifiers, code: Code) -> Result<(), String> {
        // 既存のホットキーを解除
        if let Some(old_hotkey) = self.hotkey.take() {
            self.manager
                .unregister(old_hotkey)
                .map_err(|e| format!("ホットキー解除失敗: {}", e))?;
        }

        // 新しいホットキーを作成
        let hotkey = HotKey::new(Some(modifiers), code);

        // ホットキーを登録
        self.manager
            .register(hotkey)
            .map_err(|e| format!("ホットキー登録失敗: {}", e))?;

        // ホットキーを保存
        self.hotkey = Some(hotkey);

        Ok(())
    }

    /// ホットキーイベントをポーリングして、ホットキーが押されたかチェックする
    ///
    /// # 戻り値
    /// ホットキーが押された場合は `true`、それ以外は `false`
    pub fn handle_events(&self) -> bool {
        if self.hotkey.is_none() {
            return false;
        }

        // イベントレシーバーからイベントを取得
        let receiver = GlobalHotKeyEvent::receiver();

        // すべてのイベントをチェック
        while let Ok(event) = receiver.try_recv() {
            // 登録されているホットキーのIDと一致するかチェック
            if let Some(hotkey) = &self.hotkey {
                if event.id() == hotkey.id() {
                    return true;
                }
            }
        }

        false
    }

    /// ホットキーを更新する
    ///
    /// # 引数
    /// * `modifiers` - 新しい修飾キー
    /// * `code` - 新しいキーコード
    ///
    /// # エラー
    /// ホットキーの更新に失敗した場合、エラーメッセージを返す
    pub fn update_hotkey(&mut self, modifiers: Modifiers, code: Code) -> Result<(), String> {
        self.register(modifiers, code)
    }

    /// 登録されているホットキーを取得する
    ///
    /// # 戻り値
    /// ホットキーが登録されている場合は `Some(HotKey)`、それ以外は `None`
    pub fn get_hotkey(&self) -> Option<&HotKey> {
        self.hotkey.as_ref()
    }

    /// すべてのホットキーを解除する
    ///
    /// # エラー
    /// ホットキーの解除に失敗した場合、エラーメッセージを返す
    pub fn unregister_all(&mut self) -> Result<(), String> {
        if let Some(hotkey) = self.hotkey.take() {
            self.manager
                .unregister(hotkey)
                .map_err(|e| format!("ホットキー解除失敗: {}", e))?;
        }
        Ok(())
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new().expect("デフォルトのHotkeyManagerの作成に失敗しました")
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        // 終了時にホットキーを解除
        let _ = self.unregister_all();
    }
}

/// 文字列配列から Modifiers 型に変換
/// 例: ["Ctrl", "Shift"] -> Modifiers::CONTROL | Modifiers::SHIFT
///
/// # 引数
/// * `modifiers` - 修飾キーの文字列配列
///
/// # エラー
/// 無効な修飾キーが含まれている場合、エラーメッセージを返す
///
/// # 仕様
/// - 大文字小文字を区別しない（"ctrl", "Ctrl", "CTRL" すべて許可）
/// - 対応する修飾キー: Ctrl/Control, Shift, Alt, Win/Super
/// - 重複は無視（同じ修飾キーが複数あっても1つとして扱う）
pub fn string_to_modifiers(modifiers: &[String]) -> Result<Modifiers, String> {
    let mut result = Modifiers::empty();

    for modifier in modifiers {
        let modifier_lower = modifier.to_lowercase();
        match modifier_lower.as_str() {
            "ctrl" | "control" => result |= Modifiers::CONTROL,
            "shift" => result |= Modifiers::SHIFT,
            "alt" => result |= Modifiers::ALT,
            "win" | "super" => result |= Modifiers::SUPER,
            _ => return Err(format!("無効な修飾キー: {}", modifier)),
        }
    }

    Ok(result)
}

/// 文字列から Code 型に変換
/// 例: "O" -> Code::KeyO, "F1" -> Code::F1
///
/// # 引数
/// * `key` - キーの文字列表現
///
/// # エラー
/// 無効なキーが指定された場合、エラーメッセージを返す
///
/// # 仕様
/// - 大文字小文字を区別しない
/// - 対応キー:
///   - アルファベット: A-Z → KeyA-KeyZ
///   - 数字: 0-9 → Digit0-Digit9
///   - ファンクションキー: F1-F12
///   - 特殊キー: Space, Enter, Escape, Tab, Backspace
pub fn string_to_code(key: &str) -> Result<Code, String> {
    let key_lower = key.to_lowercase();

    match key_lower.as_str() {
        // アルファベット
        "a" => Ok(Code::KeyA),
        "b" => Ok(Code::KeyB),
        "c" => Ok(Code::KeyC),
        "d" => Ok(Code::KeyD),
        "e" => Ok(Code::KeyE),
        "f" => Ok(Code::KeyF),
        "g" => Ok(Code::KeyG),
        "h" => Ok(Code::KeyH),
        "i" => Ok(Code::KeyI),
        "j" => Ok(Code::KeyJ),
        "k" => Ok(Code::KeyK),
        "l" => Ok(Code::KeyL),
        "m" => Ok(Code::KeyM),
        "n" => Ok(Code::KeyN),
        "o" => Ok(Code::KeyO),
        "p" => Ok(Code::KeyP),
        "q" => Ok(Code::KeyQ),
        "r" => Ok(Code::KeyR),
        "s" => Ok(Code::KeyS),
        "t" => Ok(Code::KeyT),
        "u" => Ok(Code::KeyU),
        "v" => Ok(Code::KeyV),
        "w" => Ok(Code::KeyW),
        "x" => Ok(Code::KeyX),
        "y" => Ok(Code::KeyY),
        "z" => Ok(Code::KeyZ),

        // 数字
        "0" => Ok(Code::Digit0),
        "1" => Ok(Code::Digit1),
        "2" => Ok(Code::Digit2),
        "3" => Ok(Code::Digit3),
        "4" => Ok(Code::Digit4),
        "5" => Ok(Code::Digit5),
        "6" => Ok(Code::Digit6),
        "7" => Ok(Code::Digit7),
        "8" => Ok(Code::Digit8),
        "9" => Ok(Code::Digit9),

        // ファンクションキー
        "f1" => Ok(Code::F1),
        "f2" => Ok(Code::F2),
        "f3" => Ok(Code::F3),
        "f4" => Ok(Code::F4),
        "f5" => Ok(Code::F5),
        "f6" => Ok(Code::F6),
        "f7" => Ok(Code::F7),
        "f8" => Ok(Code::F8),
        "f9" => Ok(Code::F9),
        "f10" => Ok(Code::F10),
        "f11" => Ok(Code::F11),
        "f12" => Ok(Code::F12),

        // 特殊キー
        "space" => Ok(Code::Space),
        "enter" => Ok(Code::Enter),
        "escape" | "esc" => Ok(Code::Escape),
        "tab" => Ok(Code::Tab),
        "backspace" => Ok(Code::Backspace),

        _ => Err(format!("無効なキー: {}", key)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotkey_manager_new() {
        let result = HotkeyManager::new();
        assert!(result.is_ok(), "HotkeyManagerの作成に失敗しました");
    }

    #[test]
    fn test_register_hotkey() {
        let mut manager = HotkeyManager::new().expect("HotkeyManagerの作成に失敗しました");

        // Ctrl+Shift+O を登録
        let modifiers = Modifiers::CONTROL | Modifiers::SHIFT;
        let code = Code::KeyO;

        let result = manager.register(modifiers, code);
        assert!(result.is_ok(), "ホットキーの登録に失敗しました: {:?}", result.err());

        // ホットキーが登録されていることを確認
        assert!(manager.get_hotkey().is_some(), "ホットキーが登録されていません");
    }

    #[test]
    fn test_update_hotkey() {
        let mut manager = HotkeyManager::new().expect("HotkeyManagerの作成に失敗しました");

        // 最初のホットキーを登録
        let modifiers1 = Modifiers::CONTROL | Modifiers::SHIFT;
        let code1 = Code::KeyO;
        manager.register(modifiers1, code1).expect("ホットキーの登録に失敗しました");

        // ホットキーを更新
        let modifiers2 = Modifiers::CONTROL | Modifiers::ALT;
        let code2 = Code::KeyP;
        let result = manager.update_hotkey(modifiers2, code2);

        assert!(result.is_ok(), "ホットキーの更新に失敗しました: {:?}", result.err());
        assert!(manager.get_hotkey().is_some(), "更新後のホットキーが登録されていません");
    }

    #[test]
    fn test_unregister_all() {
        let mut manager = HotkeyManager::new().expect("HotkeyManagerの作成に失敗しました");

        // ホットキーを登録
        let modifiers = Modifiers::CONTROL | Modifiers::SHIFT;
        let code = Code::KeyO;
        manager.register(modifiers, code).expect("ホットキーの登録に失敗しました");

        // すべてのホットキーを解除
        let result = manager.unregister_all();
        assert!(result.is_ok(), "ホットキーの解除に失敗しました: {:?}", result.err());
        assert!(manager.get_hotkey().is_none(), "ホットキーが残っています");
    }

    #[test]
    fn test_handle_events_without_registration() {
        let manager = HotkeyManager::new().expect("HotkeyManagerの作成に失敗しました");

        // ホットキーが登録されていない場合はfalseを返す
        assert!(!manager.handle_events(), "登録なしでtrueが返されました");
    }

    #[test]
    fn test_handle_events_with_registration() {
        let mut manager = HotkeyManager::new().expect("HotkeyManagerの作成に失敗しました");

        // ホットキーを登録
        let modifiers = Modifiers::CONTROL | Modifiers::SHIFT;
        let code = Code::KeyO;
        manager.register(modifiers, code).expect("ホットキーの登録に失敗しました");

        // イベントがない場合はfalseを返す
        assert!(!manager.handle_events(), "イベントなしでtrueが返されました");
    }

    #[test]
    fn test_register_multiple_times() {
        let mut manager = HotkeyManager::new().expect("HotkeyManagerの作成に失敗しました");

        // 1回目の登録
        let modifiers1 = Modifiers::CONTROL | Modifiers::SHIFT;
        let code1 = Code::KeyO;
        manager.register(modifiers1, code1).expect("1回目の登録に失敗しました");

        // 2回目の登録（上書き）
        let modifiers2 = Modifiers::CONTROL | Modifiers::ALT;
        let code2 = Code::KeyP;
        manager.register(modifiers2, code2).expect("2回目の登録に失敗しました");

        // 3回目の登録（上書き）
        let modifiers3 = Modifiers::CONTROL;
        let code3 = Code::KeyQ;
        let result = manager.register(modifiers3, code3);

        assert!(result.is_ok(), "3回目の登録に失敗しました: {:?}", result.err());
        assert!(manager.get_hotkey().is_some(), "ホットキーが登録されていません");
    }

    #[test]
    fn test_default_trait() {
        let manager = HotkeyManager::default();
        assert!(manager.get_hotkey().is_none(), "デフォルトでホットキーが登録されています");
    }

    // string_to_modifiers のテスト
    #[test]
    fn test_string_to_modifiers_normal() {
        // 正常系: "Ctrl+Shift" → Modifiers::CONTROL | SHIFT
        let modifiers = vec!["Ctrl".to_string(), "Shift".to_string()];
        let result = string_to_modifiers(&modifiers);
        assert!(result.is_ok());
        let expected = Modifiers::CONTROL | Modifiers::SHIFT;
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_string_to_modifiers_case_insensitive() {
        // 大文字小文字: "ctrl+shift" → 同じ結果
        let modifiers_lower = vec!["ctrl".to_string(), "shift".to_string()];
        let result_lower = string_to_modifiers(&modifiers_lower);
        assert!(result_lower.is_ok());

        let modifiers_upper = vec!["CTRL".to_string(), "SHIFT".to_string()];
        let result_upper = string_to_modifiers(&modifiers_upper);
        assert!(result_upper.is_ok());

        let modifiers_mixed = vec!["Ctrl".to_string(), "Shift".to_string()];
        let result_mixed = string_to_modifiers(&modifiers_mixed);
        assert!(result_mixed.is_ok());

        let expected = Modifiers::CONTROL | Modifiers::SHIFT;
        assert_eq!(result_lower.unwrap(), expected);
        assert_eq!(result_upper.unwrap(), expected);
        assert_eq!(result_mixed.unwrap(), expected);
    }

    #[test]
    fn test_string_to_modifiers_invalid() {
        // エラー系: 無効な修飾キー → Err
        let modifiers = vec!["Invalid".to_string()];
        let result = string_to_modifiers(&modifiers);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "無効な修飾キー: Invalid");
    }

    #[test]
    fn test_string_to_modifiers_all_types() {
        // すべての修飾キータイプをテスト
        let modifiers = vec![
            "Ctrl".to_string(),
            "Shift".to_string(),
            "Alt".to_string(),
            "Win".to_string(),
        ];
        let result = string_to_modifiers(&modifiers);
        assert!(result.is_ok());
        let expected = Modifiers::CONTROL | Modifiers::SHIFT | Modifiers::ALT | Modifiers::SUPER;
        assert_eq!(result.unwrap(), expected);

        // Control と Super のエイリアステスト
        let modifiers2 = vec![
            "Control".to_string(),
            "Super".to_string(),
        ];
        let result2 = string_to_modifiers(&modifiers2);
        assert!(result2.is_ok());
        let expected2 = Modifiers::CONTROL | Modifiers::SUPER;
        assert_eq!(result2.unwrap(), expected2);
    }

    #[test]
    fn test_string_to_modifiers_duplicates() {
        // 重複は無視される
        let modifiers = vec![
            "Ctrl".to_string(),
            "Ctrl".to_string(),
            "Shift".to_string(),
        ];
        let result = string_to_modifiers(&modifiers);
        assert!(result.is_ok());
        let expected = Modifiers::CONTROL | Modifiers::SHIFT;
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_string_to_modifiers_empty() {
        // 空配列の場合
        let modifiers: Vec<String> = vec![];
        let result = string_to_modifiers(&modifiers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Modifiers::empty());
    }

    // string_to_code のテスト
    #[test]
    fn test_string_to_code_alphabet() {
        // キーコード正常系: "O" → Code::KeyO
        let result = string_to_code("O");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Code::KeyO);

        // 小文字でも動作
        let result2 = string_to_code("o");
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), Code::KeyO);
    }

    #[test]
    fn test_string_to_code_invalid() {
        // キーコードエラー系: "Invalid" → Err
        let result = string_to_code("Invalid");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "無効なキー: Invalid");
    }

    #[test]
    fn test_string_to_code_digits() {
        // 数字キーのテスト
        assert_eq!(string_to_code("0").unwrap(), Code::Digit0);
        assert_eq!(string_to_code("1").unwrap(), Code::Digit1);
        assert_eq!(string_to_code("9").unwrap(), Code::Digit9);
    }

    #[test]
    fn test_string_to_code_function_keys() {
        // ファンクションキーのテスト
        assert_eq!(string_to_code("F1").unwrap(), Code::F1);
        assert_eq!(string_to_code("f1").unwrap(), Code::F1);
        assert_eq!(string_to_code("F12").unwrap(), Code::F12);
    }

    #[test]
    fn test_string_to_code_special_keys() {
        // 特殊キーのテスト
        assert_eq!(string_to_code("Space").unwrap(), Code::Space);
        assert_eq!(string_to_code("Enter").unwrap(), Code::Enter);
        assert_eq!(string_to_code("Escape").unwrap(), Code::Escape);
        assert_eq!(string_to_code("Esc").unwrap(), Code::Escape); // エイリアス
        assert_eq!(string_to_code("Tab").unwrap(), Code::Tab);
        assert_eq!(string_to_code("Backspace").unwrap(), Code::Backspace);
    }

    #[test]
    fn test_string_to_code_all_alphabet() {
        // すべてのアルファベットキーをテスト
        assert_eq!(string_to_code("A").unwrap(), Code::KeyA);
        assert_eq!(string_to_code("B").unwrap(), Code::KeyB);
        assert_eq!(string_to_code("C").unwrap(), Code::KeyC);
        assert_eq!(string_to_code("D").unwrap(), Code::KeyD);
        assert_eq!(string_to_code("E").unwrap(), Code::KeyE);
        assert_eq!(string_to_code("F").unwrap(), Code::KeyF);
        assert_eq!(string_to_code("G").unwrap(), Code::KeyG);
        assert_eq!(string_to_code("H").unwrap(), Code::KeyH);
        assert_eq!(string_to_code("I").unwrap(), Code::KeyI);
        assert_eq!(string_to_code("J").unwrap(), Code::KeyJ);
        assert_eq!(string_to_code("K").unwrap(), Code::KeyK);
        assert_eq!(string_to_code("L").unwrap(), Code::KeyL);
        assert_eq!(string_to_code("M").unwrap(), Code::KeyM);
        assert_eq!(string_to_code("N").unwrap(), Code::KeyN);
        assert_eq!(string_to_code("O").unwrap(), Code::KeyO);
        assert_eq!(string_to_code("P").unwrap(), Code::KeyP);
        assert_eq!(string_to_code("Q").unwrap(), Code::KeyQ);
        assert_eq!(string_to_code("R").unwrap(), Code::KeyR);
        assert_eq!(string_to_code("S").unwrap(), Code::KeyS);
        assert_eq!(string_to_code("T").unwrap(), Code::KeyT);
        assert_eq!(string_to_code("U").unwrap(), Code::KeyU);
        assert_eq!(string_to_code("V").unwrap(), Code::KeyV);
        assert_eq!(string_to_code("W").unwrap(), Code::KeyW);
        assert_eq!(string_to_code("X").unwrap(), Code::KeyX);
        assert_eq!(string_to_code("Y").unwrap(), Code::KeyY);
        assert_eq!(string_to_code("Z").unwrap(), Code::KeyZ);
    }
}
