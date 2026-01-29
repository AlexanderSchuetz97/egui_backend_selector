use log::LevelFilter;
use egui_backend_selector::{BackendConfiguration, BackendInterop};

struct EguiApp {}

impl EguiApp {
    fn new(context: egui::Context) -> Self {
        egui_extras::install_image_loaders(&context);
        EguiApp {}
    }
}

impl egui_backend_selector::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, backend: BackendInterop<'_>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("Hello World! Running on {}", backend.backend_name()));
        });
    }

    fn on_exit(&mut self) {
        println!("EXIT CALLED");
    }
}

fn main() {
    _= trivial_log::init_std(LevelFilter::Trace);

    egui_backend_selector::run_app("app-name", BackendConfiguration::default(), |e| EguiApp::new(e))
        .expect("failed to run app");
}