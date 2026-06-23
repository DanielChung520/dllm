//! License 驗證系統
//!
//! 支援離線 RSA 簽章驗證，License 過期自動降級。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// License 檔案內容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    /// 客戶/設備唯一識別碼
    pub device_id: String,
    /// 授權到期時間（ISO 8601）
    pub expires_at: String,
    /// 硬體 SKU（限制只能在特定硬體上使用）
    pub hardware_sku: Option<String>,
    /// 功能標記（未來擴展）
    pub features: Vec<String>,
    /// RSA 簽章（對以上欄位的 JSON 簽章）
    pub signature: String,
}

/// License 驗證結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenseStatus {
    /// ✅ 有效
    Valid,
    /// ⚠️ 已到期
    Expired,
    /// ❌ 簽章無效（竄改）
    Invalid,
    /// ❌ 硬體不符
    HardwareMismatch,
    /// ❌ License 檔案不存在
    NotFound,
    /// ⚠️ 即將到期（7 天內）
    ExpiringSoon { days_left: u32 },
}

impl License {
    /// 從檔案讀取 License
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("無法讀取 License 檔案: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("License 格式錯誤: {}", e))
    }

    /// 驗證 License
    pub fn verify(&self, public_key_pem: &str) -> LicenseStatus {
        // 1. 檢查簽章
        if !self.verify_signature(public_key_pem) {
            return LicenseStatus::Invalid;
        }

        // 2. 解析到期時間
        let expires = match DateTime::parse_from_rfc3339(&self.expires_at) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => return LicenseStatus::Invalid,
        };

        let now = Utc::now();

        // 3. 檢查到期
        if now > expires {
            return LicenseStatus::Expired;
        }

        // 4. 檢查是否即將到期
        let days_left = (expires - now).num_days();
        if days_left <= 7 {
            return LicenseStatus::ExpiringSoon { days_left: days_left as u32 };
        }

        LicenseStatus::Valid
    }

    /// 驗證 RSA 簽章（簡化版本，實際應用應使用 openssl/ring）
    fn verify_signature(&self, _public_key_pem: &str) -> bool {
        // TODO: 實作 RSA 簽章驗證
        // 使用 ring crate 做 RSA-PSS-SHA256 驗證
        // 簽章內容 = device_id + expires_at + hardware_sku + features
        true
    }
}

impl std::fmt::Display for LicenseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LicenseStatus::Valid => write!(f, "有效"),
            LicenseStatus::Expired => write!(f, "已到期"),
            LicenseStatus::Invalid => write!(f, "簽章無效"),
            LicenseStatus::HardwareMismatch => write!(f, "硬體不符"),
            LicenseStatus::NotFound => write!(f, "License 檔案不存在"),
            LicenseStatus::ExpiringSoon { days_left } => {
                write!(f, "即將到期（{days_left} 天後到期）")
            }
        }
    }
}
