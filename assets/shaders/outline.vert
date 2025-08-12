#version 450 core

layout(location = 0) in vec3 position;
layout(location = 3) in vec4 color;
layout(location = 5) in mat4 model; // takes up 4 attribute locations

out vec4 v_color;

layout (std140, binding = 1, column_major) uniform matrix_block {
    mat4 projection;
    mat4 view;
    vec4 cam_pos;
};

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    v_color = color;
}
