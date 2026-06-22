mod frame;
mod color;
mod buffer_object;
mod math;
mod vertex_array_object;
mod draw_elements_indirect_command;
mod draw_call;

use std::ptr::eq;
use gl::types::GLintptr;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::NamedKey::Control;
use crate::buffer_object::{create_buffer_object, BufferRequest};
use crate::draw_call::{DrawCall, DrawCallType};
use crate::draw_call::DrawCallType::MultiDrawElementsIndirectCount;
use crate::draw_elements_indirect_command::{DrawElementsIndirectCommand, DrawElementsIndirectCommandManager};
use crate::frame::{Application, VitreousRSHandler};
use crate::vertex_array_object::create_vertex_array_object;

fn main() {
    let window_data = frame::WindowData{
        title: String::from("Test Window"),
        ..Default::default()
    };

    let mut application = Application::new(window_data, ControlFlow::Poll, TestApplicationHandler);
    application.run();
}

pub struct TestApplicationHandler;
impl VitreousRSHandler for TestApplicationHandler {
    fn init(&self) {
        println!("VitreousRS: 初期化を開始します。");
        
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

        let mut mega_buffer = match create_buffer_object(alignment, &requests) {
            Ok(buffer) => buffer,
            Err(err) => {
                eprintln!("メガバッファの生成に失敗しました: {}", err);
                return;
            }
        };
        println!("メガバッファの生成に成功しました。ID: {}", mega_buffer.id);

        let mut sub_buffer = &mut mega_buffer.sub_buffers[0];

        let mut write_offset = 0;

        let vertex_data = [
            0.0f32,  0.5f32, 0.0f32,  1.0f32, 0.0f32, 0.0f32,
            -0.5f32, -0.5f32, 0.0f32,  0.0f32, 1.0f32, 0.0f32,
            0.5f32, -0.5f32, 0.0f32,  0.0f32, 0.0f32, 1.0f32,
        ];

        println!("GPU永続マップメモリへデータを直接ストリーミング中...");

        for &value in &vertex_data {
            if let Err(e) = sub_buffer.put_f32(&mut write_offset, value) {
                eprintln!("データ書き込みエラー: {}", e);
                return;
            }
        }

        println!("書き込み完了。消費バイト数: {} bytes", write_offset);

        let index_data = [
            1,2,3
        ];

        sub_buffer = &mut mega_buffer.sub_buffers[1];
        write_offset = 0;
        for &value in &index_data {
            if let Err(e) = sub_buffer.put_u32(&mut write_offset, value) {
                eprintln!("Writing data error!")
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

        if let Err(e) =  manager.flush_to_sub_buffer(sub_buffer,&mut write_offset) {
            eprintln!("Writing data error!")
        }

        mega_buffer.update_mega_buffer();
        println!("GPUへのメモリ同期（Flush）が完了しました。");

        let vertex_array_object = create_vertex_array_object(&mega_buffer);
        // vertex_array_object.add_vertex_buffer_object_attribute(&mega_buffer,0,)

        DrawCall::new(MultiDrawElementsIndirectCount {
            mode: gl::TRIANGLES,
            index_type: gl::UNSIGNED_INT,
            indirect_offset: mega_buffer.sub_buffers[2].offset as GLintptr,
            drawcount_offset: 0,
            max_draw_count: 10000,
            stride: 0,
        }).execute();
    }

    fn render(&self) {

    }

    fn exit(&self) {
        println!("exit");
    }
}