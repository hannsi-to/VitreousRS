use std::fmt::Alignment;
use std::fs::ReadDir;
use crate::math::align;

pub struct BufferRequest {
    pub target: u32,
    pub requested_size: usize,
    pub requested_gap_size: usize,
}

pub fn create_buffer_object(alignment: usize, requests: &[BufferRequest]) -> Result<MegaBufferObjectData, String> {
    let mut total_size: usize = 0;
    let mut offsets: Vec<usize> = Vec::new();
    let mut actual_gaps: Vec<usize> = Vec::new();

    for req in requests {
        let mut current_offset = total_size;
        align(&mut current_offset, alignment)?;

        let padding = current_offset - total_size;
        offsets.push(current_offset);

        let total_gap = req.requested_gap_size + padding;
        actual_gaps.push(total_gap);

        total_size = current_offset + req.requested_size + req.requested_gap_size;
    }

    let mut id: u32 = 0;
    let address: *mut std::ffi::c_void;
    let storage_flags = gl::DYNAMIC_STORAGE_BIT | gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT;
    let access_flags = gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT;

    unsafe {
        gl::CreateBuffers(1, &mut id);
        gl::NamedBufferStorage(id, total_size as gl::types::GLsizeiptr, std::ptr::null(), storage_flags);
        address = gl::MapNamedBufferRange(id, 0, total_size as gl::types::GLsizeiptr, access_flags);

        if address.is_null() {
            return Err(String::from("Failed to secure memory for OpenGL."));
        }
    }

    let mut mega_data = MegaBufferObjectData::new(id, address, total_size, alignment);
    for (i, req) in requests.iter().enumerate() {
        mega_data.add_sub_buffer(
            req.target,
            offsets[i],
            req.requested_size,
            actual_gaps[i],
        );
    }

    Ok(mega_data)
}

struct MegaBufferObjectData {
    id: u32,
    address: *mut std::ffi::c_void,
    size: usize,
    alignment: usize,
    pub need_reallocate: bool,
    pub sub_buffers: Vec<SubBuffers>,
    updated_memory_block: Vec<MemoryBlock>
}

unsafe impl Send for MegaBufferObjectData {}
unsafe impl Sync for MegaBufferObjectData {}

impl MegaBufferObjectData {
    pub fn new(id: u32, address: *mut std::ffi::c_void, size: usize,alignment: usize) -> MegaBufferObjectData {
        Self {
            id,
            address,
            size,
            alignment,
            need_reallocate: false,
            sub_buffers: Vec::new(),
            updated_memory_block: Vec::new(),
        }
    }

    pub fn add_sub_buffer(&mut self,target: u32, offset: usize, size: usize, gap_size: usize) {
        self.sub_buffers.push(SubBuffers {
            parent_address: self.address,
            need_reallocate: self.need_reallocate,
            target,
            offset,
            size,
            gap_size,
        });
    }

    pub fn update_reallocate_status(&mut self) {
        if self.sub_buffers.iter().any(|sub| sub.need_reallocate) {
            self.need_reallocate = true;
        }
    }
}

pub struct SubBuffers{
    parent_address: *mut std::ffi::c_void,
    pub need_reallocate: bool,
    target: u32,
    offset: usize,
    size: usize,
    gap_size: usize,
}

impl SubBuffers{
    fn check_bounded(&mut self, write_offset: usize, data_size: usize) -> Result<(), String> {
        let total_requested = write_offset + data_size;
        let total_capacity = self.size + self.gap_size;

        if total_requested > total_capacity {
            return Err(format!(
                "The buffer capacity has been exceeded perfectly! SubBuffer Offset: {} | Allowed Max: {}",
                self.offset, total_capacity
            ));
        }

        if total_requested > self.size {
            self.need_reallocate = true;
        }

        Ok(())
    }

    pub fn put_f32(&mut self,offset: &mut usize,value: f32) -> Result<(), String> {
        let f32_size = size_of::<f32>();
        self.check_bounded(*offset, f32_size)?;

        unsafe {
            let raw_target_ptr = self.parent_address.add(*offset);
            let float_target_ptr = raw_target_ptr as *mut f32;
            *float_target_ptr = value;
        }

        *offset += 4;
        Ok(())
    }
}

struct MemoryBlock{
    offset: usize,
    size: usize,
}