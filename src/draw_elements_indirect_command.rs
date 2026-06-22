use crate::buffer_object::SubBuffers;

#[repr(C)]
pub struct DrawElementsIndirectCommand {
    pub count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub base_instance: u32,
}

pub struct DrawElementsIndirectCommandManager{
    pub draw_elements_indirect_commands: Vec<DrawElementsIndirectCommand>,
}

impl DrawElementsIndirectCommandManager {
    pub fn new() -> DrawElementsIndirectCommandManager {
        Self{
            draw_elements_indirect_commands: Vec::new(),
        }
    }

    pub fn add_command(&mut self, draw_elements_indirect_command: DrawElementsIndirectCommand) {
        self.draw_elements_indirect_commands.push(draw_elements_indirect_command);
    }

    pub fn flush_to_sub_buffer(&self, sub_buffer: &mut SubBuffers, offset: &mut usize) -> Result<(), String> {
        for cmd in &self.draw_elements_indirect_commands {
            sub_buffer.put_u32(offset, cmd.count)?;
            sub_buffer.put_u32(offset, cmd.instance_count)?;
            sub_buffer.put_u32(offset, cmd.first_index)?;
            sub_buffer.put_i32(offset, cmd.base_vertex)?;
            sub_buffer.put_u32(offset, cmd.base_instance)?;
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.draw_elements_indirect_commands.len()
    }
}