//! API Key 管理與驗證

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;

const API_KEYS_FILE: &str = ".dllm/api_keys.json";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiKeyEntry {
    pub key_hash: String,
    pub label: String,
    pub created_at: String,
    pub enabled: bool,
    pub rate_limit_rpm: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiKeyStoreData {
    pub keys: Vec<ApiKeyEntry>,
    pub revoked: Vec<String>,
}

pub struct ApiKeyStore {
    file_path: PathBuf,
    data: Arc<RwLock<ApiKeyStoreData>>,
}

impl ApiKeyStore {
    pub fn new() -> Self {
        let path = Self::default_path();
        let data = Self::load_or_default(&path);
        Self { file_path: path, data: Arc::new(RwLock::new(data)) }
    }

    fn default_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(API_KEYS_FILE)
    }

    fn load_or_default(path: &PathBuf) -> ApiKeyStoreData {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| {
                let default = ApiKeyStoreData { keys: vec![], revoked: vec![] };
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(path, serde_json::to_string_pretty(&default).unwrap_or_default());
                default
            })
    }

    fn save(&self) {
        let data = self.data.read();
        let _ = std::fs::write(&self.file_path, serde_json::to_string_pretty(&*data).unwrap_or_default());
    }

    fn hash_key(raw: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(raw.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn create_key(&self, label: &str) -> String {
        let raw_key = format!("dllm-{}", Uuid::new_v4());
        let hash = Self::hash_key(&raw_key);
        let entry = ApiKeyEntry {
            key_hash: hash,
            label: label.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            enabled: true,
            rate_limit_rpm: 60,
        };
        {
            let mut data = self.data.write();
            data.keys.push(entry);
        }
        self.save();
        raw_key
    }

    pub fn revoke_key(&self, key_hash: &str) -> bool {
        let mut data = self.data.write();
        if let Some(pos) = data.keys.iter().position(|k| k.key_hash == key_hash) {
            let entry = data.keys.remove(pos);
            data.revoked.push(entry.key_hash.clone());
            self.save();
            return true;
        }
        false
    }

    pub fn validate_key(&self, raw_key: &str) -> Option<ApiKeyEntry> {
        let hash = Self::hash_key(raw_key);
        let data = self.data.read();
        data.keys.iter().find(|k| k.key_hash == hash && k.enabled).cloned()
    }

    pub fn list_keys(&self) -> Vec<(String, ApiKeyEntry)> {
        let data = self.data.read();
        data.keys.iter().map(|k| ("active".to_string(), k.clone())).collect()
    }
}
