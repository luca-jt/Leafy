#version 450 core

in vec2 v_uv;

layout(location = 1) uniform vec4 color;
layout(location = 7) uniform sampler2D tex_sampler;

void main() {
    vec4 textured = texture(tex_sampler, v_uv).rgba * color;
    if (textured.a < 0.01) {
        discard;
    }
    // gl_FragDepth = gl_FragCoord.z;
}
