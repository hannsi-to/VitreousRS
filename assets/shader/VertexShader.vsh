#version 460 core
precision mediump float;

layout(location=0) in vec3 in_position;
layout(location=1) in vec4 in_color;
layout(location=2) in vec2 in_uv;
layout(location=3) in vec3 in_normal;

struct ObjectStruct {
    uint instance_struct_index;
    uint paddint_1;
    uint paddint_2;
    uint paddint_3;
};

struct InstanceStruct {
    mat4 transform;
};

void main(){

}