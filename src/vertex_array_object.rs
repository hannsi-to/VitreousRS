use gl::types::{GLboolean, GLint, GLintptr, GLsizeiptr};
use crate::buffer_object::MegaBufferObjectData;

pub fn create_vertex_array_object(mega_buffer_object_data: &MegaBufferObjectData) -> VertexArrayObject {
    let mut id: u32 = 0;
    unsafe {
        gl::CreateVertexArrays(1, &mut id);
    }

    VertexArrayObject::new(id, mega_buffer_object_data.id)
}

pub struct VertexArrayObject {
    pub(crate) id: u32,
    mega_buffer_object_data_id: u32
}

impl VertexArrayObject {
    pub fn new(id: u32,mega_buffer_object_data_id: u32) -> VertexArrayObject {
        Self {
            id,
            mega_buffer_object_data_id,
        }
    }

    fn check_mega_buffer_object_data_id(&self, mega_buffer_object_data: &MegaBufferObjectData) -> Result<(), String>{
        if self.mega_buffer_object_data_id != mega_buffer_object_data.id {
            return Err(String::from("The MegaBufferObjectDataId registered in the VertexArrayObject does not match the MegaBufferObjectDataId passed as an argument."));
        }

        Ok(())
    }

    pub fn add_vertex_buffer_object_attribute(&self, mega_buffer_object_data: &MegaBufferObjectData, buffer_object_index: usize, index: u32, size: i32, type_ :u32, normalized: u8, stride: i32, relative_offset: u32) -> Result<(),String>{
        self.check_mega_buffer_object_data_id(&mega_buffer_object_data)?;

        let sub_buffer = mega_buffer_object_data.sub_buffers.get(buffer_object_index).ok_or_else(|| format!("SubBuffer index out of bounds: specified {}, but max len is {}.", buffer_object_index, mega_buffer_object_data.sub_buffers.len()))?;

        unsafe {
            gl::EnableVertexArrayAttrib(self.id, index);
            gl::VertexArrayAttribFormat(self.id, index, size, type_, normalized, relative_offset);
            gl::VertexArrayAttribBinding(self.id,index,index);
            gl::VertexArrayVertexBuffer(self.id, index, self.mega_buffer_object_data_id, sub_buffer.offset as GLintptr, stride);
        }

        Ok(())
    }

    pub fn connect_index_buffer_object(&self, mega_buffer_object_data: &MegaBufferObjectData) -> Result<(), String> {
        self.check_mega_buffer_object_data_id(mega_buffer_object_data)?;

        unsafe {
            gl::VertexArrayElementBuffer(self.id, self.mega_buffer_object_data_id);
        }

        Ok(())
    }

    pub fn bind_buffer_object(&self, mega_buffer_object_data: &MegaBufferObjectData, buffer_object_index: usize,binding_point: u32) -> Result<(), String> {
        self.check_mega_buffer_object_data_id(mega_buffer_object_data)?;

        let sub_buffer = mega_buffer_object_data.sub_buffers.get(buffer_object_index).ok_or_else(|| format!("SubBuffer idnex out bounds: specified {}, but max len is {}.", buffer_object_index, mega_buffer_object_data.sub_buffers.len()))?;

        unsafe {
            gl::BindBufferRange(sub_buffer.target,binding_point,self.mega_buffer_object_data_id,sub_buffer.offset as GLintptr,sub_buffer.size as GLsizeiptr);
        }

        Ok(())
    }
}