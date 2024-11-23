#version 450 core

#define MAX_LIGHT_SRC_COUNT 5

in vec2 v_uv;

out vec4 out_color;

layout(location = 1) uniform vec4 color;
layout(location = 7) uniform sampler2D tex_sampler;

void main() {
    vec4 textured = texture(tex_sampler, v_uv).rgba;
    if (textured.a < 0.01) {
        discard;
    }
    out_color = textured * color;
}
