#version 450 core

#define MAX_DIR_LIGHT_MAPS 5
#define MAX_POINT_LIGHT_MAPS 5
#define SHADOW_MAP_COUNT MAX_POINT_LIGHT_MAPS + MAX_DIR_LIGHT_MAPS

in vec4 frag_pos;
in vec4 g_color;
in vec2 g_uv;
flat in float g_tex_idx;

layout(location = 6) uniform sampler2D tex_sampler[32 - SHADOW_MAP_COUNT];

layout(location = 55) uniform vec3 light_pos;
layout(location = 55) uniform float far_plane;

void main() {
    int sampler_idx = int(round(g_tex_idx));
    vec4 textured = texture(tex_sampler[sampler_idx], g_uv).rgba * g_color;
    if (textured.a < 0.001) {
        discard;
    }

    float light_distance = length(frag_pos.xyz - light_pos);
    light_distance /= far_plane;
    gl_FragDepth = light_distance;
}
