# Egui Backend Selector

Backend selector for egui that will select a backend at runtime that works on the system your application is running on.

## Motivation
The 'default' backend for rich client egui applications is eframe,
eframe has some pretty steep system requirements (OpenGL) that not all runtimes meet.
Notably, eframe is known to NOT work on windows in:
* RDP (Remote Desktop Protocol) session
* VMWare virtual machines
* VirtualBox virtual machines
* Installations with only VGA graphics (no drivers installed yet.)

This is not ideal for developing portable applications, especially if those applications 
target Windows servers or applications that are supposed to work on virgin windows installs
without gpu drivers installed.

Thankfully, DGriffin91 implemented a software renderer (https://github.com/DGriffin91/egui_software_backend) 
for egui which works on nearly all platforms;
however, since it is not hardware accelerated, 
it is going to be slower for systems that have a gpu and proper drivers installed.

This crate contains all the platform-specific logic necessary to determine which backend
will work at runtime and will then delegate the execution of your app to that backend.
To achieve this, the crate provides a thin "drop-in" wrapper for eframe::App as well
as the equivalent of DGriffin91's software backend.

## Example

```rust
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
}

fn main() {
    _= trivial_log::init_std(LevelFilter::Trace);

    egui_backend_selector::run_app("app-name", BackendConfiguration::default(), |e| EguiApp::new(e))
        .expect("failed to run app");
}
```
## Which backend is selected on which platform?
### macOS or BSD like FreeBSD
* Always eframe

### Linux
* On wayland eframe is always chosen.
* On X11 eframe is chosen unless the current display is a remote display (like with X11 over SSH), then the software backend is chosen.

### Windows
* In the case of an RDP Session, the software backend is always chosen.
  The registry keys that determine if the dedicated graphics device "should" be used to accelerate the RDP session
  and provide an opengl version newer than opengl 1.3 are not evaluated. 
  Even if those keys are set, there is no way to know if the system even has a gpu capable of doing it.

* On X86_64 or X86 targets if the system runs in a virtual machine, then
  if the system has drivers installed that indicate VirtualBox or VMWare, the software backend is chosen.
  When attempting to launch your eframe application with any of those drivers installed, 
  your application is likely to run into an ACCESS_VIOLATION due to buggy drivers.
  * Note: These checks are *NOT* foolproof as they hard-code the name of the mentioned drivers which can change at any time,
    and also do not check if those drivers are actually loaded, as that would require system debugging privileges.
  * KVM: The KVM drivers appear to work with eframe, so the presence of KVM is not checked.
  * Microsoft HyperV: Completely untested.

* If the opengl version on the system is lower than 3.2 the software backend is chosen.
  To check this, the crate creates a small offscreen opengl context which is then discarded again.