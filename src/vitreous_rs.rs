use std::sync::{Mutex, RwLock, RwLockReadGuard};
use std::sync::OnceLock;
use crate::gl_call;

pub const API_NAME: &str = "VitreousRS";
pub const DEFAULT_PROJECT_NAME: &str = concat!("VitreousRS", " Project");

static GL_VERSION: OnceLock<(u32, u32)> = OnceLock::new();

pub fn init_gl_version() {
    GL_VERSION.get_or_init(|| {
        let ptr = gl_call!(GetString(gl::VERSION),default: std::ptr::null_mut());

        let version_str = unsafe {
            std::ffi::CStr::from_ptr(ptr as *const _)
                .to_string_lossy()
                .into_owned()
        };

        let mut parts = version_str.split('.');
        let major = parts.next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        let minor = parts.next()
            .and_then(|s| s.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse::<u32>().ok())
            .unwrap_or(0);

        (major, minor)
    });
}

pub fn get_gl_version() -> (u32, u32) {
    *GL_VERSION.get().unwrap_or(&(0, 0))
}

pub fn is_gl_version_supported(major: u32, minor: u32) -> bool {
    let (current_major, current_minor) = get_gl_version();
    (current_major, current_minor) >= (major, minor)
}

pub struct VitreousRS {
    pub project_name: &'static str,
}

impl Default for VitreousRS {
    fn default() -> Self {
        Self {
            project_name: DEFAULT_PROJECT_NAME,
        }
    }
}