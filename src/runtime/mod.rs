pub mod engine;

#[cfg(target_arch = "wasm32")]
pub mod wasm_engine;

#[cfg(target_arch = "wasm32")]
pub use wasm_engine::WasmEngine;
