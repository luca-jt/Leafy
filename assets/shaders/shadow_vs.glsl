#version 450 core

in vec3 position;
layout (location = 5) in vec3 offset; // jank but who cares for now
layout (location = 6) in float scale; // "

uniform mat4 light_matrix;
uniform mat4 model;

void main() {
    float s = scale == 0.0 ? 1.0 : scale;
    gl_Position = light_matrix * model * vec4(position * s + offset, 1.0);
}
