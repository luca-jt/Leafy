#version 450 core

in vec3 position;
in vec4 color;
in vec2 uv;
in vec3 normal;
in float tex_idx;

out vec4 v_color;
out vec2 v_uv;
out vec3 v_normal;
out float v_tex_idx;
out vec3 frag_pos;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    v_color = color;
    v_uv = uv;
    v_normal = normal;
    v_tex_idx = tex_idx;
    frag_pos = vec3(model * vec4(position, 1.0));
}
