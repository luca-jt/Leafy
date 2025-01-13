#version 450 core

layout(location = 0) in vec3 tex_coords;

out vec3 tex_coords_v;

layout (std140, binding = 1, column_major) uniform matrix_block {
    mat4 projection;
    mat4 view;
    vec4 cam_pos;
};

void main() {
    tex_coords_v = tex_coords;
    vec4 position = projection * mat4(mat3(view)) * vec4(tex_coords, 1.0);
    gl_Position = position.xyww;
}
