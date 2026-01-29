#[cfg(all(not(feature = "glow"), not(feature = "wgpu")))]
compile_error!("Either glow or wgpu feature must be enabled for eframe to be useful.");

use std::collections::HashMap;
use eframe::egui::Context;
use eframe::{Frame, IntegrationInfo, NativeOptions, Storage};
use egui_software_backend::{SoftwareBackend, SoftwareBackendAppConfiguration};
use std::error::Error;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::AtomicUsize;
use log::error;

const NUM_BACKENDS: usize = 2;
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum Backend {
    SoftwareBackend,
    Eframe
}

//0 - not decided
//1 - SoftwareBackend not launched
//2 - Eframe not launched
//3 - SoftwareBackend launched
//4 - Eframe launched
static STATE: AtomicUsize = AtomicUsize::new(0);

/// Overwrites the selected backend.
/// This has no effect if the application was already launched.
pub fn overwrite_backend(backend: Backend) {
    let state = STATE.load(Relaxed);
    if state > NUM_BACKENDS {
        return;
    }

    match backend {
        Backend::SoftwareBackend => {
            _ = STATE.compare_exchange(state, 1, Relaxed, Relaxed);
        }
        Backend::Eframe => {
            _ = STATE.compare_exchange(state, 2, Relaxed, Relaxed);
        }
    }
}

/// Returns true if the application was already launched and the selected backend can no longer be changed
/// by calling the `overwrite_backend` function.
pub fn is_launched() -> bool {
    STATE.load(Relaxed) > NUM_BACKENDS
}

/// The function returns the backend selected to be used for egui.
/// # Returns
/// This function may return None if called outside the main thread. The specific conditions
/// as to when this occurs are platform-specific and subject to change.
///
/// This function is guaranteed to never return None if it's called in the main thread.
///
pub fn get_backend() -> Option<Backend> {
    let state = STATE.load(Relaxed);
    Some(match state {
        1 => Backend::SoftwareBackend,
        2 => Backend::Eframe,
        3 => Backend::SoftwareBackend,
        4 => Backend::Eframe,
        _=> {
            return match determine_backend() {
                None => None,
                Some(Backend::SoftwareBackend) => {
                    _= STATE.compare_exchange(0, 1, Relaxed, Relaxed);
                    Some(Backend::SoftwareBackend)
                },
                Some(Backend::Eframe) => {
                    _= STATE.compare_exchange(0, 2, Relaxed, Relaxed);
                    Some(Backend::Eframe)
                }
            }
        }
    })
}


/// Platform-specific interop to interact with the backend
#[non_exhaustive]
pub enum BackendInterop<'a> {
    SoftwareBackend(SoftwareBackendInterop<'a>),
    Eframe(&'a mut Frame),
}

/// Wrapper for the SoftwareBackend
pub struct SoftwareBackendInterop<'a> {
    swb: &'a mut SoftwareBackend,
    integration_info: &'a mut IntegrationInfo,
    storage: &'a mut Option<Box<dyn Storage>>
}

impl Deref for SoftwareBackendInterop<'_> {
    type Target = SoftwareBackend;

    fn deref(&self) -> &Self::Target {
        self.swb
    }
}

impl DerefMut for SoftwareBackendInterop<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.swb
    }
}

impl BackendInterop<'_> {

    pub fn backend(&self) -> Backend {
        match self {
            BackendInterop::SoftwareBackend(_) => Backend::SoftwareBackend,
            BackendInterop::Eframe(_) => Backend::Eframe
        }
    }

    pub fn backend_name(&self) -> &'static str {
        match self {
            BackendInterop::SoftwareBackend(_) => "Software Backend",
            BackendInterop::Eframe(_) => "eframe",
        }
    }

    pub fn is_web(&self) -> bool {
        //We don't run on the web yet at all...
        false
    }

    pub fn into(&self) -> &IntegrationInfo {
        match self {
            BackendInterop::SoftwareBackend(swbi) => {
                &swbi.integration_info
            }
            BackendInterop::Eframe(efr) => {
                efr.info()
            }
        }
    }

    pub fn storage(&self) -> Option<&dyn Storage> {
        match self {
            BackendInterop::SoftwareBackend(swbi) => {
                swbi.storage.as_ref().map(Box::as_ref)
            },
            BackendInterop::Eframe(efr) => efr.storage(),
        }
    }

    pub fn storage_mut(&mut self) -> Option<&mut (dyn Storage + 'static)> {
        match self {
            BackendInterop::SoftwareBackend(swbi) => {
                swbi.storage.as_mut().map(Box::as_mut)
            },
            BackendInterop::Eframe(efr) => efr.storage_mut(),
        }
    }

    #[cfg(feature = "glow")]
    pub fn gl(&mut self) -> Option<&std::sync::Arc<eframe::glow::Context>> {
        match self {
            BackendInterop::SoftwareBackend(_) => None,
            BackendInterop::Eframe(efr) => efr.gl(),
        }
    }

    #[cfg(feature = "glow")]
    pub fn register_native_glow_texture(&mut self, native: eframe::glow::Texture) -> egui::TextureId {
        match self {
            BackendInterop::SoftwareBackend(_) => egui::TextureId::User(0), //DUMMY
            BackendInterop::Eframe(efr) => efr.register_native_glow_texture(native),
        }
    }
}

/// App traits
pub trait App {

    /// The update loop
    fn update(&mut self, context: &Context, backend: BackendInterop<'_>);


    /// This function is called once when the application exists.
    /// It is NOT called when using eframe with the wgpu backend.
    fn on_exit(&mut self) {}

    /// This function is called before on_exit and allows you to save state
    /// It might be called periodically too
    fn save(&mut self, storage: &mut dyn Storage) {
        _= storage;
    }
}

struct AppWrapper<T: App>(T, Option<Box<dyn Storage>>, IntegrationInfo);

impl <T: App> eframe::App for AppWrapper<T> {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.0.update(ctx, BackendInterop::Eframe(frame));
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        self.0.save(storage);
    }

    #[cfg(feature = "glow")]
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.0.on_exit()
    }

    #[cfg(not(feature = "glow"))]
    fn on_exit(&mut self) {
        self.0.on_exit()
    }


}
impl <T: App> egui_software_backend::App for AppWrapper<T> {
    fn update(&mut self, ctx: &Context, software_backend: &mut SoftwareBackend) {
        self.2.cpu_usage = software_backend.last_frame_time().map(|a| a.as_secs_f32());

        self.0.update(ctx, BackendInterop::SoftwareBackend(SoftwareBackendInterop {
            swb: software_backend,
            integration_info: &mut self.2,
            storage: &mut self.1,
        }));
    }

    fn on_exit(&mut self, _ctx: &Context) {
        if let Some(store) = self.1.as_mut() {
            self.0.save(store.as_mut());
            store.flush();
        }

        self.0.on_exit();
    }


}

#[derive(Default, Clone)]
pub struct BackendConfiguration {
    viewport: egui::ViewportBuilder,
    eframe_options: Option<NativeOptions>,
    software_backend_options: Option<SoftwareBackendAppConfiguration>
}


impl From<egui::ViewportBuilder> for BackendConfiguration {
    fn from(value: egui::ViewportBuilder) -> Self {
        BackendConfiguration {
            viewport: value,
            eframe_options: None,
            software_backend_options: None
        }
    }
}

impl From<NativeOptions> for BackendConfiguration {
    fn from(value: NativeOptions) -> Self {
        BackendConfiguration {
            viewport: value.viewport.clone(),
            eframe_options: Some(value),
            software_backend_options: None,
        }
    }
}


impl From<SoftwareBackendAppConfiguration> for BackendConfiguration {
    fn from(value: SoftwareBackendAppConfiguration) -> Self {
        BackendConfiguration {
            viewport: value.viewport_builder.clone(),
            
            eframe_options: None,
            software_backend_options: Some(value)
        }
    }
}


#[cfg(feature = "persistence")]
struct KVStroage {
    ron_file: PathBuf,
    kv: HashMap<String, String>,
    dirty: bool
}

#[cfg(feature = "persistence")]
fn write_ron(ron_path: impl AsRef<Path>, kvs: &HashMap<String, String>) {
    let rp = ron_path.as_ref();

    if let Some(parent) = rp.parent() && !parent.exists() {
        _ = std::fs::create_dir_all(parent);
    }

    let Ok(file) = std::fs::File::create(rp)
        .inspect_err(|e| error!("Failed to save application state. Could not create file {} err={e}", rp.display())) else {
        return;
    };

    let mut writer = std::io::BufWriter::new(file);
    if let Err(e) =  ron::Options::default()
        .to_io_writer_pretty(&mut writer, kvs, ron::ser::PrettyConfig::default()) {
        error!("Failed to save application state. Could not write file {} err={e}", rp.display())
    }
}

#[cfg(feature = "persistence")]
impl KVStroage {
    pub fn new(app_name: String) -> Option<Self> {
        let data_dir = eframe::storage_dir(&app_name)?;
        let ron_file = data_dir.join("app.ron");

        let initial_data = if ron_file.exists() {
            let file = std::fs::File::open(&ron_file).inspect_err(|e| {
                error!("Failed to read application state. Could not read file {} err={e}", ron_file.display());
            }).ok()?;

            let reader = std::io::BufReader::new(file);
            ron::de::from_reader(reader).inspect_err(|e| {
                error!("Failed to read application state. File contains invalid data {} err={e}", ron_file.display());
            }).ok()?
        } else {
            HashMap::new()
        };

        Some(KVStroage {
            ron_file,
            kv: initial_data,
            dirty: false,
        })
    }
}

#[cfg(feature = "persistence")]
impl Storage for KVStroage {
    fn get_string(&self, key: &str) -> Option<String> {
        self.kv.get(key).cloned()
    }

    fn set_string(&mut self, key: &str, value: String) {
        self.kv.insert(key.to_string(), value);
        self.dirty = true;
    }

    fn flush(&mut self) {
        if self.dirty {
            write_ron(&self.ron_file, &self.kv);
            self.dirty = false;
        }

    }
}

pub fn run_app<T: App>(app_name: &str, backend_configuration: impl Into<BackendConfiguration>, mut app_factory: impl FnMut(Context) -> T) -> Result<(), Box<dyn Error>> {
    if Some(false) == is_main_thread::is_main_thread() {
        return Err("Current thread is not the main thread".into());
    }

    if is_launched() {
        return Err("Application already launched".into());
    }

    let config = backend_configuration.into();
    match get_backend() {
        None | Some(Backend::SoftwareBackend) => {
            STATE.store(3, Relaxed);
            let mut cfg_to_use = config.software_backend_options.unwrap_or_else(|| SoftwareBackendAppConfiguration::default());
            cfg_to_use.viewport_builder = config.viewport;

            let app_name = app_name.to_string();

            if let Err(e) = egui_software_backend::run_app_with_software_backend(cfg_to_use, move |ctx| {
                #[cfg(feature = "persistence")]
                let storage :  Option<Box<dyn Storage>> = KVStroage::new(app_name.clone()).map(|a| Box::new(a) as Box<dyn Storage>);

                #[cfg(not(feature = "persistence"))]
                let storage: Option<Box<dyn Storage>> = None;

                let integration_info = IntegrationInfo {
                    cpu_usage: None
                };

                AppWrapper(app_factory(ctx), storage, integration_info)
            }) {
                return Err(Box::new(e));
            }

            Ok(())
        }
        Some(Backend::Eframe) => {
            STATE.store(4, Relaxed);
            let mut cfg_to_use = config.eframe_options.unwrap_or_else(|| NativeOptions::default());
            cfg_to_use.viewport = config.viewport;

            let integration_info = IntegrationInfo {
                cpu_usage: None
            };

            if let Err(e) = eframe::run_native(app_name, cfg_to_use, Box::new(move |ctx| Ok(Box::new(AppWrapper(app_factory(ctx.egui_ctx.clone()), None, integration_info))))) {
                return Err(Box::new(e));
            }

            Ok(())
        }
    }
}

#[cfg(all(not(windows), not(target_os = "linux")))]
fn determine_backend() -> Option<Backend> {
    //macOS and BSD.
    Some(Backend::Eframe)
}

#[cfg(target_os = "linux")]
fn determine_backend() -> Option<Backend> {
    //We only care about remote display sessions here, because eframe performs poorly on those.

    let Ok(display) = std::env::var("DISPLAY") else {
        //We are not on X11, must be wayland where eframe works.
        //I don't have any experience with waypipe (wayland via ssh) TODO test this?
        return Some(Backend::Eframe);
    };

    if !display.starts_with(":") && !display.contains("/unix:") {
        //This is remote X11 session. OpenGL will be the slowest thing in the universe.
        return Some(Backend::SoftwareBackend)
    }

    //We could check if opengl is present, however nearly all linux distros nowadays come with at least mesa llvm-pipe.
    //TODO think about this.

    Some(Backend::Eframe)
}

#[cfg(windows)]
fn determine_backend() -> Option<Backend> {
    if Some(false) == is_main_thread::is_main_thread() {
        return None;
    }

    if unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
            windows_sys::Win32::UI::WindowsAndMessaging::SM_REMOTESESSION,
        ) != 0
    } {
        //Technically, we could query some obscure registry keys here,
        //as well as some group policies. It is technically possible to enable opengl 3.2 via RDP,
        //however, it is so poorly documented by microsoft that I only managed to do it once by accident and could never reproduce it.
        //Needless to say, if it's an RDP connection, then we just use the software renderer.
        return Some(Backend::SoftwareBackend);
    }

    //We dont need to check this on aarch64 as I am pretty sure that only KVM supports this properly and the virtio drivers for it actually work with eframe.
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        if raw_cpuid::CpuId::new().get_hypervisor_info().is_some() {
            //We are running on a VM.

            // These checks cover sensible use cases.
            // I.e., They assume that it's unlikely someone migrated a VM from VirtualBox to KVM,
            // without first uninstalling all the VirtualBox drivers.
            // We could list the actual device drivers, but windows decided to remove that
            // Windows api function because of "security" with Windows 11 24H2.
            // I don't want to parse the stdout output of "querydriver.exe" yet.

            //Process will segfault if we try eframe. This is the VMWare 3d driver. It's not good enough.
            if std::fs::exists("C:\\Windows\\System32\\vm3dgl64.dll").unwrap_or(false) {
                return Some(Backend::SoftwareBackend);
            }

            //Eframe will fail to launch due to missing gl extensions. This is the Virtualbox opengl driver.
            if std::fs::exists("C:\\Windows\\System32\\VBoxGL.dll").unwrap_or(false) {
                return Some(Backend::SoftwareBackend);
            }

            //TODO test if it works with Hyper-V? (I never tested if eframe works on there by default)

            //Probably KVM? if so theres a high chance that eframe works.
        }
    }



    let Ok(mut glfw) = glfw::init::<()>(None) else {
        //No opengl at all, this is some virgin post-installer windows with no drivers.
        return Some(Backend::SoftwareBackend);
    };

    //The minimum version for eframe to work appears to be opengl 3.2
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 2));
    glfw.window_hint(glfw::WindowHint::Visible(false));

    let Some((wnd, events)) = glfw
        .create_window(128, 128, "opengl version detector", glfw::WindowMode::Windowed) else {
        //Opengl is too old. This is a catch-all for "other" hypervisors with insufficient opengl implementations.
        return Some(Backend::SoftwareBackend);
    };

    drop(events);
    drop(wnd);

    Some(Backend::Eframe)
}
