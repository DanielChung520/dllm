//! dllm-shared
//!
//! 提供 dllm 全系統共享的類型定義、trait、錯誤處理與工具函式。
//! 此 crate 無平台相依性，可在所有目標平台上編譯。

pub mod engine;
pub mod error;
pub mod license;
pub mod memory;
pub mod model;
pub mod types;

// 重新導出常用類型
pub use engine::*;
pub use error::*;
pub use memory::*;
pub use model::*;
pub use types::*;
