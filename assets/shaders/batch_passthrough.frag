#version 450 core

in vec4 v_color;
in vec2 v_uv;
in float v_tex_idx;

out vec4 out_color;

uniform sampler2D tex_sampler[27];

void main() {
    int sampler_idx = int(round(v_tex_idx));
    out_color = texture(tex_sampler[sampler_idx], v_uv).rgba * v_color;
}
