use eframe::Storage;
use log::{info};
use egui_backend_selector::{BackendInterop};

pub struct EguiApp {
    data: String
}

impl EguiApp {
    pub fn new(context: egui::Context, storage: Option<&dyn Storage>) -> Self {
        egui_extras::install_image_loaders(&context);
        let data = storage.map(|storage| {
            storage.get_string("payload").unwrap_or_default()
        }).unwrap_or_default();


        EguiApp {
            data
        }
    }
}

impl egui_backend_selector::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, backend: BackendInterop<'_>) {

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("Hello World! Running on {}", backend.backend_name()));
            ui.label("Persistent String:");
            ui.text_edit_singleline(&mut self.data);
        });
    }

    fn on_exit(&mut self) {
        info!("EXIT CALLED");
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        info!("SAVE CALLED");
        storage.set_string("payload", self.data.clone());
    }
}