mod frame;
mod color;
mod buffer_object;
mod math;
mod vertex_array_object;
mod draw_elements_indirect_command;
mod draw_call;
mod logger;
mod vitreous_rs;
mod shade;
mod shader_manager;
mod object;
mod vertex;

use std::sync::RwLock;
use gl::types::{GLintptr, GLsizei};
use glam::{vec3, Mat4};
use winit::event_loop::ControlFlow;
use crate::buffer_object::{create_buffer_object, BufferRequest, BufferSyncMode, MegaBufferObjectData};
use crate::draw_call::{DrawCall, DrawCallType};
use crate::draw_elements_indirect_command::{DrawElementsIndirectCommand, DrawElementsIndirectCommandManager};
use crate::frame::{Application, VitreousRSHandler};
use crate::logger::logger_manager::LoggerManager;
use crate::shader_manager::ShaderManager;
use crate::vertex_array_object::{create_vertex_array_object, VertexArrayObject};
use crate::vitreous_rs::VitreousRS;

fn main() {
    let vitreous_rs = VitreousRS { ..Default::default() };
    let window_data = frame::WindowData {
        title: String::from("Test Window"),
        ..Default::default()
    };

    let mut application = Application::new(
        vitreous_rs,
        window_data,
        ControlFlow::Poll,
        TestApplicationHandler::new(),
    );
    application.run();
}

struct ObjectStructData {
    instance_struct_index: u32,
    padding_1: u32,
    padding_2: u32,
    padding_3: u32,
}

// ObjectStructのデータ
struct InstanceStructData {
    transform: [f32; 16],  // mat4
}

// ✅ 状態を保持する構造体
pub struct TestApplicationHandler {
    mega_buffer:          Option<MegaBufferObjectData>,
    vertex_array_object:  Option<VertexArrayObject>,
    shader_manager:       ShaderManager,
    draw_count:           i32,
    indirect_offset:      GLintptr,
}

impl TestApplicationHandler {
    pub fn new() -> Self {
        Self {
            mega_buffer:         None,
            vertex_array_object: None,
            shader_manager:      ShaderManager::new(),
            draw_count:          0,
            indirect_offset:     0,
        }
    }
}

impl VitreousRSHandler for TestApplicationHandler {
    fn init(&mut self) {
        self.shader_manager.load_from_file(
            "Shader1",
            "assets/shader/VertexShader.vsh",
            "assets/shader/FragmentShader.fsh",
        ).unwrap();

        let requests = vec![
            BufferRequest { target: gl::ARRAY_BUFFER,         requested_size: 1024, requested_gap_size: 512 },
            BufferRequest { target: gl::ELEMENT_ARRAY_BUFFER, requested_size: 1024, requested_gap_size: 512 },
            BufferRequest { target: gl::DRAW_INDIRECT_BUFFER, requested_size: 1024, requested_gap_size: 512 },
        ];

        let mut mega_buffer = create_buffer_object(16, &requests, BufferSyncMode::Coherent).unwrap();

        // 頂点データ
        let mut write_offset = 0;
        let vertex_data = [
            0.0f32,  0.5, 0.0,  1.0, 0.0, 0.0, 1.0,
            -0.5f32, -0.5, 0.0,  0.0, 1.0, 0.0, 1.0,
            0.5f32, -0.5, 0.0,  0.0, 0.0, 1.0, 1.0,
        ];
        for &v in &vertex_data {
            mega_buffer.sub_buffers[0].put_f32(&mut write_offset, v).unwrap();
        }

        // インデックスデータ
        let index_data = [0u32, 1, 2];
        write_offset = 0;
        for &v in &index_data {
            mega_buffer.sub_buffers[1].put_u32(&mut write_offset, v).unwrap();
        }

        // IndirectCommand
        let mut manager = DrawElementsIndirectCommandManager::new();
        manager.add_command(DrawElementsIndirectCommand {
            count:          3,
            instance_count: 1,
            first_index:    (mega_buffer.sub_buffers[1].offset / size_of::<u32>()) as u32,
            base_vertex:    0,
            base_instance:  0,
        });
        write_offset = 0;
        manager.flush_to_sub_buffer(&mut mega_buffer.sub_buffers[2], &mut write_offset).unwrap();

        mega_buffer.update_mega_buffer();

        let vao = create_vertex_array_object(&mega_buffer);

        vao.add_vertex_buffer_object_attribute(
            &mega_buffer, 0, 0, 0, 3, gl::FLOAT, gl::FALSE as u8,
            (7 * size_of::<f32>()) as i32, 0,
        ).unwrap();

        vao.add_vertex_buffer_object_attribute(
            &mega_buffer, 0, 0, 1, 4, gl::FLOAT, gl::FALSE as u8,
            (7 * size_of::<f32>()) as i32,
            (3 * size_of::<f32>()) as u32,
        ).unwrap();

        vao.connect_index_buffer_object(&mega_buffer).unwrap();
        vao.bind_draw_indirect_buffer(&mega_buffer).unwrap();
        vao.bind();

        self.indirect_offset     = mega_buffer.sub_buffers[2].offset as GLintptr;
        self.draw_count          = manager.draw_elements_indirect_commands.len() as i32;
        self.mega_buffer         = Some(mega_buffer);
        self.vertex_array_object = Some(vao);

        self.shader_manager.bind("Shader1").unwrap();
    }

    fn render(&self) {
        DrawCall::new(DrawCallType::MultiDrawElementsIndirect {
            mode:            gl::TRIANGLES,
            index_type:      gl::UNSIGNED_INT,
            indirect_offset: self.indirect_offset,
            draw_count:      self.draw_count,
            stride:          0,
        }).execute();
    }

    fn exit(&self) {
        println!("exit");
    }
}