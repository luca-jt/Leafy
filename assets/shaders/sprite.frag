#version 450 core

in vec4 v_color;
in vec2 v_uv;
flat in float v_tex_idx;

out vec4 out_color;

layout(location = 0) uniform sampler2D tex_sampler[32];

void main() {
    int sampler_idx = int(round(v_tex_idx));
    vec4 textured = texture(tex_sampler[sampler_idx], v_uv).rgba;
    if (textured.a < 0.001) {
        discard;
    }
    out_color = textured * v_color;
}
