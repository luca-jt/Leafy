#version 450 core

in vec2 v_uv;

out vec4 out_color;

layout(location = 1) uniform vec4 color;
layout(location = 2) uniform sampler2D tex_sampler;
layout(location = 3) uniform bool transparent_pass;

void main() {
    vec4 textured = texture(tex_sampler, v_uv).rgba;
    if (textured.a < 0.01 || (textured.a < 0.99 && !transparent_pass) || (textured.a >= 0.99 && transparent_pass)) {
        discard;
    }
    out_color = textured * color;
}
