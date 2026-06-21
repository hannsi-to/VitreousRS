use std::cmp::{max, min};
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

#[derive(Clone)]
pub struct MegaBufferObjectData {
    pub(crate) id: u32,
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
        let flag_ptr: *mut bool = &mut self.need_reallocate as *mut bool;

        self.sub_buffers.push(SubBuffers {
            parent_address: self.address,
            parent_need_reallocate_ptr: flag_ptr,
            target,
            offset,
            size,
            gap_size,
            updated_memory_blocks: Vec::new(),
        });
    }

    pub fn update_mega_buffer(&mut self) {
        for sub_buffer in &mut self.sub_buffers {
            for block in &sub_buffer.updated_memory_blocks {
                unsafe {
                    gl::FlushMappedNamedBufferRange(
                        self.id,
                        block.offset as gl::types::GLintptr,
                        block.size as gl::types::GLsizeiptr,
                    );
                }
            }
            sub_buffer.updated_memory_blocks.clear();
        }
    }
}

#[derive(Clone)]
pub struct SubBuffers{
    parent_address: *mut std::ffi::c_void,
    parent_need_reallocate_ptr: *mut bool,
    target: u32,
    pub offset: usize,
    pub size: usize,
    pub gap_size: usize,
    updated_memory_blocks: Vec<MemoryBlock>
}

impl SubBuffers{
    const MAX_ALLOW_GAP_SIZE: usize = 64;

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
            unsafe {
                if !self.parent_need_reallocate_ptr .is_null() {
                    *self.parent_need_reallocate_ptr = true;
                }
            }
        }

        Ok(())
    }

    fn update_memory_block(&mut self, offset: &mut usize,size: usize) {
        let mut new_start: usize = self.offset + *offset;
        let mut new_end = self.offset + *offset + size;

        let mut i: usize = 0;
        while i < self.updated_memory_blocks.len() {
            let current_block = self.updated_memory_blocks[i];
            let current_start = current_block.offset;
            let current_end = current_block.offset + current_block.size;

            let mut gap: usize = 0;
            if new_start > current_end {
                gap = new_start - current_end;
            } else if current_start > new_end {
                gap = current_start - new_end;
            }

            if (new_start <= current_end && new_end >= current_start) || gap <= Self::MAX_ALLOW_GAP_SIZE {
                new_start = min(new_start, current_start);
                new_end = max(new_end, current_end);

                self.updated_memory_blocks.remove(i);
                continue;
            }

            if current_start < new_start {
                i += 1;
            } else {
                break;
            }
        }

        self.updated_memory_blocks.insert(i, MemoryBlock {
            offset: new_start,
            size: new_end - new_start,
        });

        *offset += size;
    }

    pub fn put_i32(&mut self,offset: &mut usize, value: i32) -> Result<(), String> {
        let i32_size = size_of::<i32>();
        self.check_bounded(*offset, i32_size)?;

        unsafe {
            let raw_target_ptr = self.parent_address.add(self.offset + *offset);
            let int_target_ptr = raw_target_ptr as *mut i32;
            *int_target_ptr = value;
        }

        self.update_memory_block(offset, i32_size);

        Ok(())
    }

    pub fn put_f32(&mut self,offset: &mut usize,value: f32) -> Result<(), String> {
        let f32_size = size_of::<f32>();
        self.check_bounded(*offset, f32_size)?;

        unsafe {
            let raw_target_ptr = self.parent_address.add(self.offset + *offset);
            let float_target_ptr = raw_target_ptr as *mut f32;
            *float_target_ptr = value;
        }

        self.update_memory_block(offset, f32_size);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct MemoryBlock{
    offset: usize,
    size: usize,
}