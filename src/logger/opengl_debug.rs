use crate::{debug_ln, error_ln, info_ln, warning_ln};
use gl::types::*;
use std::collections::HashMap;
use std::panic;
use crate::frame::{Application, VitreousRSHandler};
use crate::logger::logger_manager::LoggerManager;

pub fn get_opengl_debug(severity_types: Vec<u32>, logger_manager: Option<&LoggerManager>) {
    let data = Box::new(DebugCallbackData {
        severity_types,
        logger_manager,
    });
    let user_param = Box::into_raw(data) as *mut std::ffi::c_void;

    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::DebugMessageCallback(Some(debug_callback), user_param);
    }
}

struct DebugCallbackData<'a> {
    severity_types: Vec<GLenum>,
    logger_manager: Option<&'a LoggerManager>,
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
        data.logger_manager.unwrap().error_logging(format!("{}", log_message).as_str());
        panic!("OpenGL Error: {}", message);
    } else if severity == gl::DEBUG_SEVERITY_HIGH {
        data.logger_manager.unwrap().error_logging(format!("{}", log_message).as_str());
    } else if severity == gl::DEBUG_SEVERITY_MEDIUM {
        data.logger_manager.unwrap().warning_logging(format!("{}", log_message).as_str());
    } else if severity == gl::DEBUG_SEVERITY_LOW {
        data.logger_manager.unwrap().info_logging(format!("{}", log_message).as_str());
    } else {
        data.logger_manager.unwrap().debug_logging(format!("{}", log_message).as_str());
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