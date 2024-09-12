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
out vec4 frag_pos_light;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 light_matrix;

void main() {
    gl_Position = projection * view * vec4(position, 1.0); // model matrix is already calculated in
    v_color = color;
    v_uv = uv;
    v_normal = normal;
    v_tex_idx = tex_idx;
    frag_pos = position;
    frag_pos_light = light_matrix * vec4(frag_pos, 1.0);
}
