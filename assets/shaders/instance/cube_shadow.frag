#version 450 core

in vec4 frag_pos;
in vec2 g_uv;

layout(location = 1) uniform vec4 color;
layout(location = 7) uniform sampler2D tex_sampler;

layout(location = 55) uniform vec3 light_pos;
layout(location = 55) uniform float far_plane;

void main() {
    vec4 textured = texture(tex_sampler, g_uv).rgba * color;
    if (textured.a < 0.001) {
        discard;
    }

    float light_distance = length(frag_pos.xyz - light_pos);
    light_distance /= far_plane;
    gl_FragDepth = light_distance;
}
