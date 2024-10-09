#version 450 core

in vec3 position;
in vec2 uv;
in vec3 normal;
layout (location = 5) in mat4 model; // takes up 4 attribute locations

out vec4 v_color;
out vec2 v_uv;
out vec3 v_normal;

layout (std140, binding = 0, column_major) uniform matrix_block {
    mat4 projection;
    mat4 view;
};

uniform vec4 color;
uniform int num_lights;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    v_color = color;
    v_uv = uv;
    v_normal = normal;
}
