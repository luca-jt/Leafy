#version 450 core

layout(location = 0) in vec3 position;
layout (location = 5) in mat4 model; // takes up 4 attribute locations

layout(location = 33) uniform mat4 light_matrix;
layout(location = 34) uniform int use_input_model;

void main() {
    mat4 input_model = use_input_model == 1 ? model : mat4(1.0);
    gl_Position = light_matrix * input_model * vec4(position, 1.0);
}
