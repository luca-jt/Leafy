#version 450 core

in vec3 position;
layout (location = 5) in mat4 model; // takes up 4 attribute locations

uniform mat4 light_matrix;
uniform mat4 general_model;
uniform int use_input_model;

void main() {
    mat4 input_model = use_input_model == 1 ? model : mat4(1.0);
    gl_Position = light_matrix * input_model * general_model * vec4(position, 1.0);
}
