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

use std::ptr::eq;
use std::sync::RwLock;
use gl::types::GLintptr;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::NamedKey::Control;
use crate::buffer_object::{create_buffer_object, BufferRequest, BufferSyncMode};
use crate::draw_call::{DrawCall, DrawCallType};
use crate::draw_call::DrawCallType::MultiDrawElementsIndirectCount;
use crate::draw_elements_indirect_command::{DrawElementsIndirectCommand, DrawElementsIndirectCommandManager};
use crate::frame::{Application, VitreousRSHandler};
use crate::logger::logger_manager::LoggerManager;
use crate::logger::opengl_debug::get_opengl_debug;
use crate::shader_manager::ShaderManager;
use crate::vertex_array_object::create_vertex_array_object;
use crate::vitreous_rs::{VitreousRS};

fn main() {
    let vitreous_rs = VitreousRS {
        ..Default::default()
    };

    let window_data = frame::WindowData{
        title: String::from("Test Window"),
        ..Default::default()
    };

    let mut application = Application::new(vitreous_rs, window_data, ControlFlow::Poll, TestApplicationHandler);
    application.run();
}

pub struct TestApplicationHandler;
impl VitreousRSHandler for TestApplicationHandler {
    fn init(&self) {
        let mut shader_manager = ShaderManager::new();
        shader_manager.load_from_file(
            "Shader1",
            "assets/shader/VertexShader.vsh",
            "assets/shader/FragmentShader.fsh",
        ).unwrap();

        shader_manager.bind("Shader1").unwrap();

        LoggerManager::debug_logging("VitreousRS: 初期化を開始します。");

        let requests = vec![
            BufferRequest {
                target: gl::ARRAY_BUFFER,
                requested_size: 1024,
                requested_gap_size: 512,
            },
            BufferRequest {
                target: gl::ELEMENT_ARRAY_BUFFER,
                requested_size: 1024,
                requested_gap_size: 512,
            },
            BufferRequest {
                target: gl::DRAW_INDIRECT_BUFFER,
                requested_size: 1024,
                requested_gap_size: 512,
            }
        ];

        let alignment = 16;

        let mut mega_buffer = match create_buffer_object(alignment, &requests,BufferSyncMode::Coherent) {
            Ok(buffer) => buffer,
            Err(err) => {
                LoggerManager::error_logging(format!("メガバッファの生成に失敗しました: {}", err).as_str());
                return;
            }
        };
        LoggerManager::debug_logging(format!("メガバッファの生成に成功しました。ID: {}", mega_buffer.id).as_str());

        let mut sub_buffer = &mut mega_buffer.sub_buffers[0];

        let mut write_offset = 0;

        let vertex_data = [
            0.0f32,  0.5f32, 0.0f32,  1.0f32, 0.0f32, 0.0f32,
            -0.5f32, -0.5f32, 0.0f32,  0.0f32, 1.0f32, 0.0f32,
            0.5f32, -0.5f32, 0.0f32,  0.0f32, 0.0f32, 1.0f32,
        ];

        LoggerManager::debug_logging("GPU永続マップメモリへデータを直接ストリーミング中...");

        for &value in &vertex_data {
            if let Err(e) = sub_buffer.put_f32(&mut write_offset, value) {
                LoggerManager::error_logging(format!("データ書き込みエラー: {}", e).as_str());
                return;
            }
        }

        LoggerManager::debug_logging(format!("書き込み完了。消費バイト数: {} bytes", write_offset).as_str());

        let index_data = [
            1,2,3
        ];

        sub_buffer = &mut mega_buffer.sub_buffers[1];
        write_offset = 0;
        for &value in &index_data {
            if let Err(e) = sub_buffer.put_u32(&mut write_offset, value) {
                LoggerManager::error_logging("Writing data error!")
            }
        }

        let mut manager = DrawElementsIndirectCommandManager::new();
        manager.add_command(
            DrawElementsIndirectCommand {
                count: index_data.len() as u32,
                instance_count: 1,
                first_index: 0,
                base_vertex: 0,
                base_instance: 0,
            }
        );

        sub_buffer = &mut mega_buffer.sub_buffers[2];
        write_offset = 0;

        if let Err(_e) =  manager.flush_to_sub_buffer(sub_buffer, &mut write_offset) {
            LoggerManager::error_logging("Writing data error!")
        }

        mega_buffer.update_mega_buffer();
        LoggerManager::debug_logging("GPUへのメモリ同期（Flush）が完了しました。");

        let vertex_array_object = create_vertex_array_object(&mega_buffer);
        vertex_array_object.add_vertex_buffer_object_attribute(
            &mega_buffer,
            0,           // buffer_object_index（ARRAY_BUFFER）
            0,           // binding_point
            0,           // attribute index（position）
            3,           // size（xyz）
            gl::FLOAT,
            gl::FALSE as u8,
            (6 * size_of::<f32>()) as i32,  // stride
            0,           // relative_offset
        ).unwrap();

        vertex_array_object.add_vertex_buffer_object_attribute(
            &mega_buffer,
            0,           // buffer_object_index
            0,           // binding_point
            1,           // attribute index（color）
            3,           // size（rgb）
            gl::FLOAT,
            gl::FALSE as u8,
            (6 * size_of::<f32>()) as i32,  // stride
            (3 * size_of::<f32>()) as u32,  // relative_offset（xyzの後）
        ).unwrap();

        // インデックスバッファの接続
        vertex_array_object.connect_index_buffer_object(&mega_buffer).unwrap();

        // DrawCall前にVAOをバインド（シーン切り替え時など）
        vertex_array_object.bind();

        // DrawCall::new(MultiDrawElementsIndirectCount {
        //     mode: gl::TRIANGLES,
        //     index_type: gl::UNSIGNED_INT,
        //     indirect_offset: mega_buffer.sub_buffers[2].offset as GLintptr,
        //     drawcount_offset: 0,
        //     max_draw_count: 10000,
        //     stride: 0,
        // }).execute();

    }

    fn render(&self) {
        // LoggerManager::debug_logging("A".repeat(100).as_str());
    }

    fn exit(&self) {
        println!("exit");
    }
}