#version 450 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;
layout(location = 5) in mat4 model; // takes up 4 attribute locations

out vec2 v_uv;

void main() {
    gl_Position = model * vec4(position, 1.0);
    v_uv = uv;
}
