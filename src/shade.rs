use std::ffi::CString;
use gl::types::GLuint;
use crate::gl_call;
use crate::logger::logger_manager::LoggerManager;

#[derive(Debug, Clone, Copy)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Geometry,
    Compute,
    TessControl,
    TessEvaluation,
}

impl ShaderType {
    pub fn to_gl(&self) -> gl::types::GLenum {
        match self {
            ShaderType::Vertex          => gl::VERTEX_SHADER,
            ShaderType::Fragment        => gl::FRAGMENT_SHADER,
            ShaderType::Geometry        => gl::GEOMETRY_SHADER,
            ShaderType::Compute         => gl::COMPUTE_SHADER,
            ShaderType::TessControl     => gl::TESS_CONTROL_SHADER,
            ShaderType::TessEvaluation  => gl::TESS_EVALUATION_SHADER,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ShaderType::Vertex          => "Vertex",
            ShaderType::Fragment        => "Fragment",
            ShaderType::Geometry        => "Geometry",
            ShaderType::Compute         => "Compute",
            ShaderType::TessControl     => "TessControl",
            ShaderType::TessEvaluation  => "TessEvaluation",
        }
    }
}

pub struct Shader {
    pub id:          GLuint,
    pub shader_type: ShaderType,
}

impl Shader {
    fn compile_shader(source: &mut str) -> Result<&str, String> {
        Ok("")
    }

    pub fn from_source(shader_type: ShaderType, source: &str) -> Result<Self, String> {
        let compiled_source = Self::compile_shader(source)?;

        let id = gl_call!(CreateShader(shader_type.to_gl()), default: 0);

        if id == 0 {
            return Err(format!("Failed to create {} shader.", shader_type.name()));
        }

        let c_source = CString::new(compiled_source).map_err(|e| format!("Invalid shader source: {}", e))?;

        gl_call!(ShaderSource(id, 1, &c_source.as_ptr(), std::ptr::null()));
        gl_call!(CompileShader(id));

        let mut success = 0;
        gl_call!(GetShaderiv(id, gl::COMPILE_STATUS, &mut success));

        if success == 0 {
            let mut len = 0;
            gl_call!(GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len));

            let mut log = vec![0u8; len as usize];
            gl_call!(GetShaderInfoLog(
                id, len, std::ptr::null_mut(),
                log.as_mut_ptr() as *mut gl::types::GLchar,
            ));

            let error = String::from_utf8_lossy(&log).to_string();
            gl_call!(DeleteShader(id));

            return Err(format!(
                "{} shader compilation failed:\n{}",
                shader_type.name(), error
            ));
        }

        LoggerManager::debug_logging(
            &format!("{} shader compiled successfully. (ID: {})", shader_type.name(), id)
        );

        Ok(Self { id, shader_type })
    }

    pub fn from_file(shader_type: ShaderType, path: &str) -> Result<Self, String> {
        let source = std::fs::read_to_string(path).map_err(|e| format!("Failed to read shader file '{}': {}", path, e))?;
        Self::from_source(shader_type, &source)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        gl_call!(DeleteShader(self.id));
        LoggerManager::debug_logging(
            &format!("Shader deleted. (ID: {})", self.id)
        );
    }
}