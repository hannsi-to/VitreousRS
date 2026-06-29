use gl::types::*;
use std::panic;
use crate::logger::logger_manager::{LoggerConfig, LoggerManager};

#[macro_export]
macro_rules! ext_call {
    ($func_ptr:expr, $func_name:expr, ( $($arg:expr),* $(,)? )) => {{
        match $func_ptr.get().and_then(|f| f.as_ref()) {
            Some(func) => unsafe { func($($arg),*) },
            None => {
                $crate::logger::logger_manager::LoggerManager::warning_logging(
                    &format!(
                        "Extension function '{}' is not supported. ({}:{})",
                        $func_name, file!(), line!()
                    )
                );
            }
        }
    }};

    ($func_ptr:expr, $func_name:expr, ( $($arg:expr),* $(,)? ), fallback: $fallback:block) => {{
        match $func_ptr.get().and_then(|f| f.as_ref()) {
            Some(func) => unsafe { func($($arg),*) },
            None => {
                $crate::logger::logger_manager::LoggerManager::warning_logging(
                    &format!(
                        "Extension function '{}' is not supported. ({}:{})",
                        $func_name, file!(), line!()
                    )
                );
                $fallback
            }
        }
    }};

    ($func_ptr:expr, $func_name:expr, ( $($arg:expr),* $(,)? ), default: $default:expr) => {{
        match $func_ptr.get().and_then(|f| f.as_ref()) {
            Some(func) => unsafe { func($($arg),*) },
            None => {
                $crate::logger::logger_manager::LoggerManager::warning_logging(
                    &format!(
                        "Extension function '{}' is not supported. ({}:{})",
                        $func_name, file!(), line!()
                    )
                );
                $default
            }
        }
    }};

    ($func_ptr:expr, $func_name:expr, ( $($arg:expr),* $(,)? ), default: $default:expr, fallback: $fallback:block) => {{
        match $func_ptr.get().and_then(|f| f.as_ref()) {
            Some(func) => unsafe { func($($arg),*) },
            None => {
                $crate::logger::logger_manager::LoggerManager::warning_logging(
                    &format!(
                        "Extension function '{}' is not supported. ({}:{})",
                        $func_name, file!(), line!()
                    )
                );
                $fallback;
                $default
            }
        }
    }};
}

#[macro_export]
macro_rules! gl_call {
    ($func:ident ( $($arg:expr),* $(,)? )) => {{
        if gl::$func::is_loaded() {
            unsafe { gl::$func($($arg),*) }
        } else {
            $crate::logger::logger_manager::LoggerManager::warning_logging(&format!(
                "OpenGL function '{}' is not supported on this platform. ({}:{})",
                stringify!($func), file!(), line!()
            ));
        }
    }};

    ($func:ident ( $($arg:expr),* $(,)? ), fallback: $fallback:block) => {{
        if gl::$func::is_loaded() {
            unsafe { gl::$func($($arg),*) }
        } else {
            $crate::logger::logger_manager::LoggerManager::warning_logging(&format!(
                "OpenGL function '{}' is not supported on this platform. ({}:{})",
                stringify!($func), file!(), line!()
            ));
            $fallback
        }
    }};

    ($func:ident ( $($arg:expr),* $(,)? ), default: $default:expr) => {{
        if gl::$func::is_loaded() {
            unsafe { gl::$func($($arg),*) }
        } else {
            $crate::logger::logger_manager::LoggerManager::warning_logging(&format!(
                "OpenGL function '{}' is not supported on this platform. ({}:{})",
                stringify!($func), file!(), line!()
            ));
            $default
        }
    }};

    ($func:ident ( $($arg:expr),* $(,)? ), default: $default:expr, fallback: $fallback:block) => {{
        if gl::$func::is_loaded() {
            unsafe { gl::$func($($arg),*) }
        } else {
            $crate::logger::logger_manager::LoggerManager::warning_logging(&format!(
                "OpenGL function '{}' is not supported on this platform. ({}:{})",
                stringify!($func), file!(), line!()
            ));
            $fallback;
            $default
        }
    }};
}

pub fn get_opengl_debug(severity_types: Vec<u32>) {
    let data = Box::new(DebugCallbackData {
        severity_types,
    });
    let user_param = Box::into_raw(data) as *mut std::ffi::c_void;

    gl_call!(Enable(gl::DEBUG_OUTPUT));
    gl_call!(Enable(gl::DEBUG_OUTPUT));
    gl_call!(Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS));
    gl_call!(DebugMessageCallback(Some(debug_callback), user_param));
}

struct DebugCallbackData {
    severity_types: Vec<GLenum>,
}

extern "system" fn debug_callback(
    source: GLenum,
    gl_type: GLenum,
    id: GLuint,
    severity: GLenum,
    _length: GLsizei,
    message: *const GLchar,
    user_param: *mut std::ffi::c_void,
) {
    let message = unsafe {
        std::ffi::CStr::from_ptr(message)
            .to_string_lossy()
            .into_owned()
    };

    let data = unsafe {
        &*(user_param as *const DebugCallbackData)
    };

    if !data.severity_types.contains(&severity) {
        return;
    }

    let source_str = get_source_string(source);
    let type_str = get_type_string(gl_type);
    let severity_str = get_severity_string(severity);

    let log_message = format!(
        "OpenGL Debug Message\nSource: {}\nType: {}\nID: {}\nSeverity: {}\nMessage: {}",
        source_str, type_str, id, severity_str, message
    );

    if gl_type == gl::DEBUG_TYPE_ERROR {
        LoggerManager::error_logging(format!("{}", log_message).as_str());
        panic!("OpenGL Error: {}", message);
    } else if severity == gl::DEBUG_SEVERITY_HIGH {
        LoggerManager::error_logging(format!("{}", log_message).as_str());
    } else if severity == gl::DEBUG_SEVERITY_MEDIUM {
        LoggerManager::warning_logging(format!("{}", log_message).as_str());
    } else if severity == gl::DEBUG_SEVERITY_LOW {
        LoggerManager::info_logging(format!("{}", log_message).as_str());
    } else {
        LoggerManager::debug_logging(format!("{}", log_message).as_str());
    }
}

fn get_source_string(source: GLenum) -> &'static str {
    match source {
        gl::DEBUG_SOURCE_API             => "API",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM   => "Window System",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "Shader Compiler",
        gl::DEBUG_SOURCE_THIRD_PARTY     => "Third Party",
        gl::DEBUG_SOURCE_APPLICATION     => "Application",
        gl::DEBUG_SOURCE_OTHER           => "Other",
        _                                => "Unknown",
    }
}

fn get_type_string(gl_type: GLenum) -> &'static str {
    match gl_type {
        gl::DEBUG_TYPE_ERROR               => "Error",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated Behavior",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR  => "Undefined Behavior",
        gl::DEBUG_TYPE_PORTABILITY         => "Portability",
        gl::DEBUG_TYPE_PERFORMANCE         => "Performance",
        gl::DEBUG_TYPE_OTHER               => "Other",
        _                                  => "Unknown",
    }
}

fn get_severity_string(severity: GLenum) -> &'static str {
    match severity {
        gl::DEBUG_SEVERITY_NOTIFICATION => "Notification",
        gl::DEBUG_SEVERITY_LOW          => "Low",
        gl::DEBUG_SEVERITY_MEDIUM       => "Medium",
        gl::DEBUG_SEVERITY_HIGH         => "High",
        _                               => "Unknown",
    }
}