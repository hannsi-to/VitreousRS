use crate::gl_call;
use crate::logger::logger_manager::LoggerManager;
use crate::vitreous_rs::get_gl_version;
use gl::types::GLuint;
use std::ffi::CString;

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
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
            ShaderType::Geometry => gl::GEOMETRY_SHADER,
            ShaderType::Compute => gl::COMPUTE_SHADER,
            ShaderType::TessControl => gl::TESS_CONTROL_SHADER,
            ShaderType::TessEvaluation => gl::TESS_EVALUATION_SHADER,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ShaderType::Vertex => "Vertex",
            ShaderType::Fragment => "Fragment",
            ShaderType::Geometry => "Geometry",
            ShaderType::Compute => "Compute",
            ShaderType::TessControl => "TessControl",
            ShaderType::TessEvaluation => "TessEvaluation",
        }
    }
}

pub struct Shader {
    pub id: GLuint,
    pub shader_type: ShaderType,
}

impl Shader {
    const fn glsl_version_from_opengl(major: u32, minor: u32) -> u32 {
        match (major, minor) {
            (2, 0) => 110,
            (2, 1) => 120,
            (3, 0) => 130,
            (3, 1) => 140,
            (3, 2) => 150,
            (3, 3) => 330,
            (4, 0) => 400,
            (4, 1) => 410,
            (4, 2) => 420,
            (4, 3) => 430,
            (4, 4) => 440,
            (4, 5) => 450,
            (4, 6) => 460,
            _ => 0,
        }
    }

    fn get_supported_glsl_version() -> u32 {
        Self::glsl_version_from_opengl(get_gl_version().0, get_gl_version().1)
    }

    fn split_with_delimiters<'a>(s: &'a str, delimiters: &[char]) -> Vec<&'a str> {
        let mut result = Vec::new();
        let mut last = 0;

        for (i, c) in s.char_indices() {
            if delimiters.contains(&c) {
                if last < i {
                    result.push(&s[last..i]);
                }
                result.push(&s[i..i + c.len_utf8()]);
                last = i + c.len_utf8();
            }
        }

        if last < s.len() {
            result.push(&s[last..]);
        }

        result
    }

    fn skip_whitespace_and_collect<'a>(
        iter: &mut std::iter::Peekable<impl Iterator<Item = &'a &'a str>>,
    ) -> String {
        let mut collected = String::new();
        while iter.peek().map(|&&t| t == " " || t == "\t") == Some(true) {
            collected.push_str(iter.next().unwrap());
        }
        collected
    }

    fn collect_until_version_directive<'a>(
        iter: &mut std::iter::Peekable<impl Iterator<Item = &'a &'a str>>,
        output: &mut String,
        collect: bool,
        supported_version: u32,
    ) -> Result<(), String> {
        let mut current_collect = collect;
        let mut depth = 0;

        loop {
            let token = match iter.next() {
                Some(&t) => t,
                None => return Err(String::from("Missing '#end_version' directive")),
            };

            if token != "#" {
                if current_collect && depth == 0 {
                    output.push_str(token);
                }
                continue;
            }

            while iter.peek().map(|&&t| t == " " || t == "\t") == Some(true) {
                iter.next();
            }

            let keyword = iter.next().copied().unwrap_or("");

            match keyword {
                "if_version" => {
                    Self::skip_whitespace_and_collect(iter);

                    let version_str = iter.next().copied().unwrap_or("");
                    let required_version: u32 = version_str.parse().unwrap_or(0);

                    while iter.peek().map(|&&t| t != "\n") == Some(true) {
                        iter.next();
                    }

                    if current_collect && depth == 0 {
                        let nested_collect = supported_version >= required_version;
                        Self::collect_until_version_directive(
                            iter,
                            output,
                            nested_collect,
                            supported_version,
                        )?;
                    } else {
                        depth += 1;
                    }
                }

                "else_version" => {
                    while iter.peek().map(|&&t| t != "\n") == Some(true) {
                        iter.next();
                    }

                    if depth == 0 {
                        current_collect = !current_collect;
                    }
                }

                "end_version" => {
                    while iter.peek().map(|&&t| t != "\n") == Some(true) {
                        iter.next();
                    }

                    if depth == 0 {
                        return Ok(());
                    } else {
                        depth -= 1;
                    }
                }

                _ => {
                    if current_collect && depth == 0 {
                        output.push('#');
                        output.push_str(keyword);
                    }
                }
            }
        }
    }

    fn compile_shader(source: &str) -> Result<String, String> {
        let tokens = Self::split_with_delimiters(
            source,
            &[
                '#', ' ', '(', ')', '=', '{', '}', '[', ']', ';', '+', '-', '*', '/', '%', ',',
                '.', '\n', '\t', '\r',
            ],
        );

        let supported_version = Self::get_supported_glsl_version();
        let mut iter = tokens.iter().peekable();
        let mut final_source = String::new();
        let mut version_found = false;
        let mut token_index = 0;

        while let Some(&token) = iter.next() {
            if token == "#" {
                let mut lookahead = String::from("#");
                lookahead.push_str(&Self::skip_whitespace_and_collect(&mut iter));

                let keyword = iter.next().copied().unwrap_or("");
                lookahead.push_str(keyword);
                match keyword {
                    "version" => {
                        lookahead.push_str(&Self::skip_whitespace_and_collect(&mut iter));

                        let version_str = match iter.next() {
                            Some(&v) => v,
                            None => {
                                return Err(String::from(
                                    "GLSL version number is missing after '#version'",
                                ));
                            }
                        };

                        if version_str == "auto" {
                            final_source.push_str(&format!("#version {} core", supported_version));

                            while iter.peek().map(|&&t| t != "\n" && t != "\r") == Some(true) {
                                iter.next();
                            }
                        } else {
                            let version: u32 = version_str.parse().map_err(|_| {
                                format!("Invalid GLSL version number: '{}'", version_str)
                            })?;

                            if version > supported_version {
                                return Err(format!(
                                    "Shader requires GLSL {}, but GPU only supports up to GLSL {}.",
                                    version, supported_version
                                ));
                            }

                            final_source.push_str(&lookahead);
                            final_source.push_str(version_str);
                        }

                        version_found = true;
                    }

                    "if_version" => {
                        Self::skip_whitespace_and_collect(&mut iter);

                        let version_str = match iter.next() {
                            Some(&v) => v,
                            None => return Err(String::from(
                                "Version number is missing after '#if_version'"
                            )),
                        };

                        let required_version: u32 = version_str.parse().map_err(|_| {
                            format!("Invalid version number after '#if_version': '{}'", version_str)
                        })?;

                        while iter.peek().map(|&&t| t != "\n") == Some(true) {
                            iter.next();
                        }

                        let collect = supported_version >= required_version;

                        Self::collect_until_version_directive(
                            &mut iter,
                            &mut final_source,
                            collect,
                            supported_version,
                        )?;
                    }

                    _ => {
                        final_source.push_str(&lookahead);
                    }
                }
            } else {
                final_source.push_str(token);
            }
        }

        if !version_found {
            return Err(String::from(
                "'#version' directive not found in shader source",
            ));
        }

        Ok(final_source)
    }

    pub fn from_source(shader_type: ShaderType, source: &str) -> Result<Self, String> {
        let compiled_source = Self::compile_shader(source)?;

        let id = gl_call!(CreateShader(shader_type.to_gl()), default: 0);

        if id == 0 {
            return Err(format!("Failed to create {} shader.", shader_type.name()));
        }

        let c_source =
            CString::new(compiled_source).map_err(|e| format!("Invalid shader source: {}", e))?;

        gl_call!(ShaderSource(id, 1, &c_source.as_ptr(), std::ptr::null()));
        gl_call!(CompileShader(id));

        let mut success = 0;
        gl_call!(GetShaderiv(id, gl::COMPILE_STATUS, &mut success));

        if success == 0 {
            let mut len = 0;
            gl_call!(GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len));

            let mut log = vec![0u8; len as usize];
            gl_call!(GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                log.as_mut_ptr() as *mut gl::types::GLchar,
            ));

            let error = String::from_utf8_lossy(&log).to_string();
            gl_call!(DeleteShader(id));

            return Err(format!(
                "{} shader compilation failed:\n{}",
                shader_type.name(),
                error
            ));
        }

        LoggerManager::debug_logging_ln(&format!(
            "{} shader compiled successfully. (ID: {})",
            shader_type.name(),
            id
        ));

        Ok(Self { id, shader_type })
    }

    pub fn from_file(shader_type: ShaderType, path: &str) -> Result<Self, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read shader file '{}': {}", path, e))?;
        Self::from_source(shader_type, &source)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        gl_call!(DeleteShader(self.id));
        LoggerManager::debug_logging_ln(&format!("Shader deleted. (ID: {})", self.id));
    }
}
