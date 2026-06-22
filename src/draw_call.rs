use std::sync::OnceLock;
use gl::types::{GLenum, GLintptr, GLsizei};

type MultiDrawElementsIndirectCountFn = unsafe extern "system" fn(
    mode: GLenum,
    type_: GLenum,
    indirect: *const std::ffi::c_void,
    drawcount: GLintptr,
    maxdrawcount: GLsizei,
    stride: GLsizei,
);

static MULTI_DRAW_INDIRECT_COUNT_FN: OnceLock<MultiDrawElementsIndirectCountFn> = OnceLock::new();

pub fn load_extensions(get_proc: impl Fn(&str) -> *const std::ffi::c_void) {
    MULTI_DRAW_INDIRECT_COUNT_FN.get_or_init(|| {
        let ptr = get_proc("glMultiDrawElementsIndirectCount");
        assert!(!ptr.is_null(), "glMultiDrawElementsIndirectCount is not supported (requires OpenGL 4.6)");
        unsafe { std::mem::transmute(ptr) }
    });
}

pub enum DrawCallType {
    MultiDrawElementsIndirectCount {
        mode: GLenum,
        index_type: GLenum,
        indirect_offset: GLintptr,
        drawcount_offset: GLintptr,
        max_draw_count: GLsizei,
        stride: GLsizei,
    },
}

pub struct DrawCall {
    pub draw_call_type: DrawCallType,
}

impl DrawCall {
    pub fn new(draw_call_type: DrawCallType) -> DrawCall {
        Self {
            draw_call_type,
        }
    }

    pub fn execute(&self) {
        let multi_draw_fn = MULTI_DRAW_INDIRECT_COUNT_FN
            .get()
            .expect("draw_call::load_extensions() must be called before DrawCall::execute()");

        unsafe {
            match &self.draw_call_type {
                DrawCallType::MultiDrawElementsIndirectCount {
                    mode,
                    index_type,
                    indirect_offset,
                    drawcount_offset,
                    max_draw_count,
                    stride,
                } => {
                    multi_draw_fn(
                        *mode,
                        *index_type,
                        *indirect_offset as *const std::ffi::c_void,
                        *drawcount_offset,
                        *max_draw_count,
                        *stride,
                    );
                }
            }
        }
    }
}
