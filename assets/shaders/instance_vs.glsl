#version 450 core

in vec3 position;
in vec2 uv;
in vec3 normal;
layout (location = 5) in vec3 offset;
layout (location = 6) in float scale;

out vec4 v_color;
out vec2 v_uv;
out vec3 v_normal;
out vec3 frag_pos;
out vec4 frag_pos_light;

uniform vec4 color;
uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;
uniform mat4 light_matrix;

void main() {
    gl_Position = projection * view * model * vec4(position * scale + offset, 1.0);
    v_color = color;
    v_uv = uv;
    v_normal = normal;
    frag_pos = vec3(model * vec4(position * scale + offset, 1.0));
    frag_pos_light = light_matrix * vec4(frag_pos + offset, 1.0);
}
