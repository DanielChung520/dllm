//! dllm-mac
//!
//! Mac 平台後端適配層。
//! 負責管理 MLX Python 子進程、監控 Metal 統一記憶體。

pub mod engine;
pub mod factory;
pub mod health;
pub mod memory;
pub mod mlx_process;

pub use engine::MLXProcessEngine;
pub use factory::MLXEngineFactory;
