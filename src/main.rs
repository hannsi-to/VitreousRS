mod frame;
mod color;
mod buffer_object;
mod math;
mod vertex_array_object;
mod draw_elements_indirect_command;

use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::NamedKey::Control;
use crate::buffer_object::{create_buffer_object, BufferRequest};
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

        let sub_buffer = &mut mega_buffer.sub_buffers[0];

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

        mega_buffer.update_mega_buffer();
        println!("GPUへのメモリ同期（Flush）が完了しました。");

        let vertex_array_object = create_vertex_array_object(&mega_buffer);
        // vertex_array_object.add_vertex_buffer_object_attribute(&mega_buffer,0,)

    }

    fn render(&self) {

    }

    fn exit(&self) {
        println!("exit");
    }
}