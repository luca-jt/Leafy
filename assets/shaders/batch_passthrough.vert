#version 450 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec2 uv;
layout(location = 4) in float tex_idx;

out vec4 v_color;
out vec2 v_uv;
flat out float v_tex_idx;

layout (std140, binding = 1, column_major) uniform matrix_block {
    mat4 projection;
    mat4 view;
    vec4 cam_pos;
};

void main() {
    gl_Position = projection * view * vec4(position, 1.0); // model matrix is already calculated in
    v_color = color;
    v_uv = uv;
    v_tex_idx = tex_idx;
}
