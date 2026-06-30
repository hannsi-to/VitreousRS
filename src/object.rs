use glam::Mat4;
use crate::vertex::Vertex;

pub struct Object {
    vertices: Vec<Vertex>,
    object_struct_data: ObjectStructData,
    instance_struct_datum: Vec<InstanceStructData>,
}

pub struct ObjectStructData {
    instance_struct_index: u32,
    padding_1: u32,
    padding_2: u32,
    padding_3: u32,
}

pub struct InstanceStructData {
    pub model_matrix: Mat4,
}

