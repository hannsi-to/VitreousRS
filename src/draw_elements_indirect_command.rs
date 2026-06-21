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

    pub fn addCommand(&mut self, draw_elements_indirect_command: DrawElementsIndirectCommand) {
        self.draw_elements_indirect_commands.push(draw_elements_indirect_command);
    }
}