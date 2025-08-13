#version 450 core

layout(location = 0) in vec3 position;
layout(location = 5) in mat4 model; // takes up 4 attribute locations
layout(location = 12) in vec4 outline_color;
layout(location = 13) in float outline_thickness;

out vec4 v_outline_color;

layout (std140, binding = 1, column_major) uniform matrix_block {
    mat4 projection;
    mat4 view;
    vec4 cam_pos;
};

void main() {
    mat4 scale = mat4(1.0 + outline_thickness);
    scale[3].w = 1;
    gl_Position = projection * view * model * scale * vec4(position, 1.0);
    v_outline_color = outline_color;
}
