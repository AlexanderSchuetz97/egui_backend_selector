#[cfg(not(target_arch = "wasm32"))]
mod implementation;

#[cfg(not(target_arch = "wasm32"))]
pub use implementation::*;