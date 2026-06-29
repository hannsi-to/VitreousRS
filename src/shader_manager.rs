use std::collections::HashMap;
use std::ptr::null_mut;
use gl::types::GLuint;
use crate::gl_call;
use crate::logger::logger_manager::LoggerManager;
use crate::shade::{Shader, ShaderType};

pub struct ShaderProgram {
    pub id: GLuint,
}

impl ShaderProgram {
    pub fn new(shaders: &[&Shader]) -> Result<Self, String> {
        let id = gl_call!(CreateProgram(), default: 0);

        if id == 0 {
            return Err(String::from("Failed to create shader program."));
        }

        for shader in shaders {
            gl_call!(AttachShader(id, shader.id));
        }

        gl_call!(LinkProgram(id));

        let mut success = 0;
        gl_call!(GetProgramiv(id, gl::LINK_STATUS, &mut success));

        if success == 0 {
            let mut len = 0;
            gl_call!(GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len));

            let mut log = vec![0u8; len as usize];
            gl_call!(GetProgramInfoLog(
                id, len, std::ptr::null_mut(),
                log.as_mut_ptr() as *mut gl::types::GLchar,
            ));

            let error = String::from_utf8_lossy(&log).to_string();
            gl_call!(DeleteProgram(id));

            return Err(format!("Shader program link failed:\n{}", error));
        }

        for shader in shaders {
            gl_call!(DetachShader(id, shader.id));
        }

        LoggerManager::debug_logging(
            &format!("Shader program linked successfully. (ID: {})", id)
        );

        Ok(Self { id })
    }

    pub fn bind(&self) {
        gl_call!(UseProgram(self.id));
    }

    pub fn unbind() {
        gl_call!(UseProgram(0));
    }

    pub fn set_uniform_i32(&self, name: &str, value: i32) -> Result<(), String> {
        let location = self.get_uniform_location(name)?;
        gl_call!(Uniform1i(location, value));
        Ok(())
    }

    pub fn set_uniform_f32(&self, name: &str, value: f32) -> Result<(), String> {
        let location = self.get_uniform_location(name)?;
        gl_call!(Uniform1f(location, value));
        Ok(())
    }

    pub fn set_uniform_vec2(&self, name: &str, x: f32, y: f32) -> Result<(), String> {
        let location = self.get_uniform_location(name)?;
        gl_call!(Uniform2f(location, x, y));
        Ok(())
    }

    pub fn set_uniform_vec3(&self, name: &str, x: f32, y: f32, z: f32) -> Result<(), String> {
        let location = self.get_uniform_location(name)?;
        gl_call!(Uniform3f(location, x, y, z));
        Ok(())
    }

    pub fn set_uniform_vec4(&self, name: &str, x: f32, y: f32, z: f32, w: f32) -> Result<(), String> {
        let location = self.get_uniform_location(name)?;
        gl_call!(Uniform4f(location, x, y, z, w));
        Ok(())
    }

    pub fn set_uniform_mat4(&self, name: &str, matrix: &[f32; 16]) -> Result<(), String> {
        let location = self.get_uniform_location(name)?;
        gl_call!(UniformMatrix4fv(location, 1, gl::FALSE, matrix.as_ptr()));
        Ok(())
    }

    fn get_uniform_location(&self, name: &str) -> Result<gl::types::GLint, String> {
        let c_name = std::ffi::CString::new(name)
            .map_err(|e| format!("Invalid uniform name '{}': {}", name, e))?;

        let location = gl_call!(GetUniformLocation(self.id, c_name.as_ptr()),default: -1);

        if location == -1 {
            return Err(format!("Uniform '{}' not found in shader program.", name));
        }

        Ok(location)
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        gl_call!(DeleteProgram(self.id));
        LoggerManager::debug_logging(
            &format!("Shader program deleted. (ID: {})", self.id)
        );
    }
}

pub struct ShaderManager {
    programs: HashMap<String, ShaderProgram>,
}

impl ShaderManager {
    pub fn new() -> Self {
        Self {
            programs: HashMap::new(),
        }
    }

    pub fn load_from_source(
        &mut self,
        name: &str,
        vertex_src: &str,
        fragment_src: &str,
    ) -> Result<(), String> {
        let vertex   = Shader::from_source(ShaderType::Vertex, vertex_src)?;
        let fragment = Shader::from_source(ShaderType::Fragment, fragment_src)?;
        let program  = ShaderProgram::new(&[&vertex, &fragment])?;

        self.programs.insert(name.to_string(), program);
        LoggerManager::info_logging(&format!("Shader '{}' loaded.", name));

        Ok(())
    }

    pub fn load_from_file(
        &mut self,
        name: &str,
        vertex_path: &str,
        fragment_path: &str,
    ) -> Result<(), String> {
        let vertex   = Shader::from_file(ShaderType::Vertex, vertex_path)?;
        let fragment = Shader::from_file(ShaderType::Fragment, fragment_path)?;
        let program  = ShaderProgram::new(&[&vertex, &fragment])?;

        self.programs.insert(name.to_string(), program);
        LoggerManager::info_logging(
            &format!("Shader '{}' loaded from files. (vert: {}, frag: {})",
                     name, vertex_path, fragment_path)
        );

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&ShaderProgram> {
        self.programs.get(name)
    }

    pub fn bind(&self, name: &str) -> Result<(), String> {
        match self.programs.get(name) {
            Some(program) => {
                program.bind();
                Ok(())
            }
            None => Err(format!("Shader '{}' not found.", name)),
        }
    }

    pub fn remove(&mut self, name: &str) {
        if self.programs.remove(name).is_some() {
            LoggerManager::info_logging(&format!("Shader '{}' removed.", name));
        }
    }
}