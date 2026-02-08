//! # Egui Backend Selector
//!
//! Backend selector for egui that will select a backend at runtime that works on the system your application is running on.
//!
//! # Example
//! ```rust
//! use egui_backend_selector::{BackendConfiguration, BackendInterop};
//! use eframe::Storage;
//!
//! struct EguiApp {}
//!
//! impl EguiApp {
//!     fn new(_context: egui::Context, _storage: Option<&dyn Storage>) -> Self {
//!         EguiApp {}
//!     }
//! }
//!
//! impl egui_backend_selector::App for EguiApp {
//!     fn update(&mut self, ctx: &egui::Context, backend: BackendInterop<'_>) {
//!         egui::CentralPanel::default().show(ctx, |ui| {
//!             ui.label(format!("Hello World! Running on {}", backend.backend_name()));
//!         });
//!     }
//! }
//!
//! fn you_main_function() {
//!     egui_backend_selector::run_app("app-name", BackendConfiguration::default(), |ctx, storage| EguiApp::new(ctx, storage))
//!         .expect("failed to run app");
//! }
//! ```

#![deny(clippy::correctness)]
#![deny(
    clippy::perf,
    clippy::complexity,
    clippy::style,
    clippy::nursery,
    clippy::pedantic,
    clippy::clone_on_ref_ptr,
    clippy::decimal_literal_representation,
    clippy::float_cmp_const,
    clippy::missing_docs_in_private_items,
    clippy::multiple_inherent_impl,
    clippy::unwrap_used,
    clippy::cargo_common_metadata,
    clippy::used_underscore_binding
)]

/// The actual implementation is found in this file.
#[cfg(not(target_arch = "wasm32"))]
mod implementation;

#[cfg(not(target_arch = "wasm32"))]
pub use implementation::*;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDocTests;
