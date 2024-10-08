#version 450 core

in vec3 position;
in vec4 color;
in vec2 uv;
in vec3 normal;
in float tex_idx;

out vec4 v_color;
out vec2 v_uv;
out float v_tex_idx;

layout (std140, binding = 0, column_major) uniform matrix_block {
    mat4 projection;
    mat4 view;
};

void main() {
    gl_Position = projection * view * vec4(position, 1.0); // model matrix is already calculated in
    v_color = color;
    v_uv = uv;
    v_tex_idx = tex_idx;
}
