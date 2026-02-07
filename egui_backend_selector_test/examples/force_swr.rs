use log::LevelFilter;
use egui_backend_selector::{Backend, BackendConfiguration};
use crate::app::EguiApp;

#[path = "app/app.rs"]
mod app;

fn main() {
    _= trivial_log::init_std(LevelFilter::Trace);

    egui_backend_selector::overwrite_backend(Backend::SoftwareBackend);

    egui_backend_selector::run_app("egui-backend-selector-test", BackendConfiguration::default(), |e, s| EguiApp::new(e, s))
        .expect("failed to run app");
}