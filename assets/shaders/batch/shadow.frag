#version 450 core

#define MAX_LIGHT_SRC_COUNT 5

in vec4 v_color;
in vec2 v_uv;
flat in float v_tex_idx;

layout(location = 6) uniform sampler2D tex_sampler[32 - MAX_LIGHT_SRC_COUNT];

void main() {
    int sampler_idx = int(round(v_tex_idx));
    vec4 textured = texture(tex_sampler[sampler_idx], v_uv).rgba * v_color;
    if (textured.a < 0.01) {
        discard;
    }
    // gl_FragDepth = gl_FragCoord.z;
}
