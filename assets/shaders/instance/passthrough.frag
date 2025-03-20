#version 450 core

in vec2 v_uv;

out vec4 out_color;

layout(location = 0) uniform vec4 color;
layout(location = 1) uniform sampler2D tex_sampler;
layout(location = 2) uniform bool transparent_pass;

void main() {
    vec4 textured = texture(tex_sampler, v_uv).rgba;
    if (textured.a < 0.001 || (textured.a < 0.999 && !transparent_pass)) {
        discard;
    }
    out_color = textured * color;
}
