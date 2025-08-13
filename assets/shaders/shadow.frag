#version 450 core

in vec2 v_uv;

layout(location = 4) uniform vec4 color;
layout(location = 5) uniform sampler2D tex_sampler;

void main() {
    vec4 textured = texture(tex_sampler, v_uv).rgba * color;
    if (textured.a < 0.5) {
        // transparent objects don't cast shadows
        discard;
    }
    // gl_FragDepth = gl_FragCoord.z;
}
