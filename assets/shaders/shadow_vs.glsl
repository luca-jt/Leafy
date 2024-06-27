#version 450 core

in vec3 position;
layout (location = 5) in vec3 offset; // jank but who cares for now

uniform mat4 light_matrix;
uniform mat4 model;

void main() {
    gl_Position = light_matrix * model * vec4(position + offset, 1.0);
}
