use std::sync::OnceLock;
use gl::types::{GLenum, GLintptr, GLsizei};
use crate::{ext_call, gl_call};
use crate::logger::logger_manager::LoggerManager;

type MultiDrawElementsIndirectCountFn = unsafe extern "system" fn(
    mode:         GLenum,
    type_:        GLenum,
    indirect:     *const std::ffi::c_void,
    drawcount:    GLintptr,
    maxdrawcount: GLsizei,
    stride:       GLsizei,
);

static MULTI_DRAW_INDIRECT_COUNT_FN: OnceLock<Option<MultiDrawElementsIndirectCountFn>> = OnceLock::new();

pub fn load_extensions(get_proc: impl Fn(&str) -> *const std::ffi::c_void) {
    MULTI_DRAW_INDIRECT_COUNT_FN.get_or_init(|| {
        let ptr = get_proc("glMultiDrawElementsIndirectCount");
        if ptr.is_null() {
            LoggerManager::warning_logging_ln(
                "glMultiDrawElementsIndirectCount is not supported. (requires OpenGL 4.6)"
            );
            None
        } else {
            Some(unsafe { std::mem::transmute::<*const std::ffi::c_void, MultiDrawElementsIndirectCountFn>(ptr) })
        }
    });
}

pub enum DrawCallType {
    MultiDrawElementsIndirectCount {
        mode:             GLenum,
        index_type:       GLenum,
        indirect_offset:  GLintptr,
        drawcount_offset: GLintptr,
        max_draw_count:   GLsizei,
        stride:           GLsizei,
    },

    MultiDrawElementsIndirect {
        mode:            GLenum,
        index_type:      GLenum,
        indirect_offset: GLintptr,
        draw_count:      GLsizei,
        stride:          GLsizei,
    },
}

pub struct DrawCall {
    pub draw_call_type: DrawCallType,
}

impl DrawCall {
    pub fn new(draw_call_type: DrawCallType) -> DrawCall {
        Self { draw_call_type }
    }

    pub fn execute(&self) {
        match &self.draw_call_type {
            DrawCallType::MultiDrawElementsIndirectCount {
                mode,
                index_type,
                indirect_offset,
                drawcount_offset,
                max_draw_count,
                stride,
            } => {
                ext_call!(
                    MULTI_DRAW_INDIRECT_COUNT_FN,
                    "glMultiDrawElementsIndirectCount",
                    (
                        *mode,
                        *index_type,
                        *indirect_offset as *const std::ffi::c_void,
                        *drawcount_offset,
                        *max_draw_count,
                        *stride,
                    ),
                    fallback: {
                        LoggerManager::warning_logging_ln(
                            "Falling back to glMultiDrawElementsIndirect."
                        );
                        gl_call!(
                            MultiDrawElementsIndirect(
                                *mode,
                                *index_type,
                                *indirect_offset as *const std::ffi::c_void,
                                *max_draw_count,
                                *stride,
                            ),
                            fallback: {
                                LoggerManager::error_logging_ln(
                                    "MultiDrawElementsIndirect is also not supported."
                                );
                            }
                        );
                    }
                );
            }

            DrawCallType::MultiDrawElementsIndirect {
                mode,
                index_type,
                indirect_offset,
                draw_count,
                stride,
            } => {
                gl_call!(
                    MultiDrawElementsIndirect(
                        *mode,
                        *index_type,
                        *indirect_offset as *const std::ffi::c_void,
                        *draw_count,
                        *stride,
                    )
                );
            }
        }
    }
}