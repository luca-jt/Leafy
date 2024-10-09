#version 450 core

#define MAX_LIGHT_SRC_COUNT 5

in vec4 v_color;
in vec2 v_uv;
in vec3 v_normal;
in float v_tex_idx;

out vec4 out_color;

uniform sampler2D shadow_sampler[MAX_LIGHT_SRC_COUNT];
uniform sampler2D tex_sampler[32 - MAX_LIGHT_SRC_COUNT];
uniform int num_lights;

void main() {
    int sampler_idx = int(round(v_tex_idx));
    out_color = texture(tex_sampler[sampler_idx], v_uv).rgba * v_color;
}
