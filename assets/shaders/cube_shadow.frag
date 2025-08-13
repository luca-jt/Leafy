#version 450 core

#define FAR_PLANE 100.0

in vec4 frag_pos;
in vec2 g_uv;

layout(location = 24) uniform vec4 color;
layout(location = 25) uniform sampler2D tex_sampler;
layout(location = 46) uniform vec3 light_pos;

void main() {
    vec4 textured = texture(tex_sampler, g_uv).rgba * color;
    if (textured.a < 0.5) {
        // transparent objects don't cast shadows
        discard;
    }

    float light_distance = length(frag_pos.xyz - light_pos);
    light_distance /= FAR_PLANE;
    gl_FragDepth = light_distance;
}
