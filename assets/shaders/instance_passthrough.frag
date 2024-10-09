#version 450 core

#define MAX_LIGHT_SRC_COUNT 5

in vec4 v_color;
in vec2 v_uv;
in vec3 v_normal;

out vec4 out_color;

uniform sampler2D shadow_sampler[MAX_LIGHT_SRC_COUNT];
uniform sampler2D tex_sampler;
uniform int num_lights;

void main() {
    out_color = texture(tex_sampler, v_uv).rgba * v_color;
}
