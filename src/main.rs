mod frame;
mod color;
mod buffer_object;
mod math;

use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::NamedKey::Control;
use crate::buffer_object::create_buffer_object;
use crate::frame::{Application, VitreousRSHandler};

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
        println!("init");
    }

    fn render(&self) {

    }

    fn exit(&self) {
        println!("exit");
    }
}