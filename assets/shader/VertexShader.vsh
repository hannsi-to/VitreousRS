#version auto core
precision mediump float;

layout(location=0) in vec3 in_position;
layout(location=1) in vec4 in_color;
//layout(location=2) in vec2 in_uv;
//layout(location=3) in vec3 in_normal;

out vec4 frag_color;
out vec2 frag_uv;

struct ObjectStruct {
    uint instance_struct_index;
    uint paddint_1;
    uint paddint_2;
    uint paddint_3;
};

struct InstanceStruct {
    mat4 transform;
};

/*
 * Buffer Strategy by GLSL Version:
 * -------------------------------------------------------
 * GLSL 430+  : SSBO (std430) + layout binding  + dynamic array []
 * GLSL 420+  : UBO  (std140) + layout binding  + fixed array [256]
 * GLSL 412+  : UBO  (std140) + ARB extension   + fixed array [256]
 * GLSL 410-  : UBO  (std140) + manual binding  + fixed array [256]
 * -------------------------------------------------------
 */
#if_version 430
    layout(std430, binding = 1) buffer ObjectStructs {
        ObjectStruct objectStructs[];
    };

    layout(std430, binding = 2) buffer InstanceStructs {
        InstanceStruct instanceStructs[];
    };
#else_version
    #if_version 420
        layout(std140, binding = 1) uniform ObjectStructsUBO {
            ObjectStruct objectStructs[256];
        };

        layout(std140, binding = 2) uniform InstanceStructsUBO {
            InstanceStruct instanceStructs[256];
        };
    #else_version
        #if_version 412
            #extension GL_ARB_shading_language_420pack : enable

            layout(std140, binding = 1) uniform ObjectStructsUBO {
                ObjectStruct objectStructs[256];
            };

            layout(std140, binding = 2) uniform InstanceStructsUBO {
                InstanceStruct instanceStructs[256];
            };
        #else_version
            layout(std140) uniform ObjectStructsUBO {
                ObjectStruct objectStructs[256];
            };

            layout(std140) uniform InstanceStructsUBO {
                InstanceStruct instanceStructs[256];
            };
        #end_version
    #end_version
#end_version

void main(){
    uint object_index    = uint(gl_DrawID);
    uint instance_index  = objectStructs[object_index].instance_struct_index;
    mat4 transform       = instanceStructs[instance_index].transform;

    gl_Position = vec4(in_position, 1.0);
    frag_color  = in_color;
}