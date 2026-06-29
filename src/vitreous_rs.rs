use crate::logger::logger_manager::LoggerManager;

pub const API_NAME: &str = "VitreousRS";
pub const DEFAULT_PROJECT_NAME: &str = concat!("VitreousRS", " Project");

pub struct VitreousRS {
    pub project_name: &'static str,
    pub logger_manager: Option<&'static LoggerManager>,
}

impl Default for VitreousRS {
    fn default() -> Self {
        Self {
            project_name: DEFAULT_PROJECT_NAME,
            logger_manager: None,
        }
    }
}