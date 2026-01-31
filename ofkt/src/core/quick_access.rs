use crate::data::models::QuickAccessEntry;
use crate::data::storage;
use anyhow::Result;
use chrono::Utc;
use std::path::PathBuf;
use uuid::Uuid;

/// クイックアクセス管理
#[derive(Debug, Clone)]
pub struct QuickAccessManager {
    entries: Vec<QuickAccessEntry>,
}

impl QuickAccessManager {
    /// 新しい QuickAccessManager を作成
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// エントリを追加
    pub fn add_entry(
        &mut self,
        name: String,
        path: PathBuf,
    ) -> Result<(), String> {
        // パスの正規化
        let canonical_path = path.canonicalize()
            .map_err(|e| format!("パスの正規化に失敗: {}", e))?;

        // 重複チェック
        if self.entries.iter().any(|e| e.path == canonical_path) {
            return Err("このフォルダは既にクイックアクセスに追加されています".to_string());
        }

        // パスが存在するかチェック
        if !canonical_path.exists() {
            return Err("指定されたパスが存在しません".to_string());
        }

        // ディレクトリかチェック
        if !canonical_path.is_dir() {
            return Err("指定されたパスはディレクトリではありません".to_string());
        }

        // 新しいエントリを作成
        let entry = QuickAccessEntry {
            id: Uuid::new_v4().to_string(),
            name,
            path: canonical_path,
            added_at: Utc::now(),
            order: self.entries.len() as u32,
            is_system: false,
        };

        self.entries.push(entry);
        Ok(())
    }

    /// IDでエントリを削除
    pub fn remove_entry_by_id(&mut self, id: &str) -> Result<(), String> {
        let index = self.entries
            .iter()
            .position(|e| e.id == id)
            .ok_or_else(|| format!("エントリID '{}' は存在しません", id))?;

        // システム項目は削除不可
        if self.entries[index].is_system {
            return Err("システム項目は削除できません".to_string());
        }

        self.entries.remove(index);

        // orderを再計算
        for (i, entry) in self.entries.iter_mut().enumerate() {
            entry.order = i as u32;
        }

        Ok(())
    }

    /// エントリ一覧を取得（order順）
    pub fn get_entries(&self) -> Vec<QuickAccessEntry> {
        let mut entries = self.entries.clone();
        entries.sort_by_key(|e| e.order);
        entries
    }

    /// ファイルに保存
    pub fn save(&self) -> Result<()> {
        storage::save_quick_access(&self.entries)
    }

    /// ファイルから読み込み
    pub fn load(&mut self) -> Result<()> {
        self.entries = storage::load_quick_access()?;
        Ok(())
    }
}

impl Default for QuickAccessManager {
    fn default() -> Self {
        Self::new()
    }
}
