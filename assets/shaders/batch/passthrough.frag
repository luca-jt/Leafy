#version 450 core

#define MAX_DIR_LIGHT_MAPS 5
#define MAX_POINT_LIGHT_MAPS 5
#define SHADOW_MAP_COUNT MAX_POINT_LIGHT_MAPS + MAX_DIR_LIGHT_MAPS

in vec4 v_color;
in vec2 v_uv;
flat in float v_tex_idx;

out vec4 out_color;

layout(location = 0) uniform bool transparent_pass;
layout(location = 11) uniform sampler2D tex_sampler[32 - SHADOW_MAP_COUNT];

void main() {
    int sampler_idx = int(round(v_tex_idx));
    vec4 textured = texture(tex_sampler[sampler_idx], v_uv).rgba;
    if (textured.a < 0.001 || (textured.a < 0.999 && !transparent_pass)) {
        discard;
    }
    out_color = textured * v_color;
}
