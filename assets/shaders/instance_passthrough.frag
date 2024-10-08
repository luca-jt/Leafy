#version 450 core

in vec4 v_color;
in vec2 v_uv;

out vec4 out_color;

uniform sampler2D tex_sampler;

void main() {
    out_color = texture(tex_sampler, v_uv).rgba * v_color;
}
