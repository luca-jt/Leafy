#version 450 core

in vec4 v_color;
in vec2 v_uv;
in vec3 v_normal;
in float v_tex_idx;
in vec3 frag_pos;
in vec4 frag_pos_light;

out vec4 out_color;

uniform sampler2D shadow_map;
uniform sampler2D tex_sampler[31];
uniform vec3 light_pos;

float shadow_calc(vec4 fpl) {
    vec3 proj_coords = fpl.xyz / fpl.w;
    proj_coords = proj_coords * 0.5 + 0.5;

    float bias = 0.001;
    int filter_size = 2;

    float shadow = 0.0;

    for (int y = -filter_size / 2; y < filter_size / 2; ++y) {
        for (int x = -filter_size / 2; x < filter_size / 2; ++x) {
            vec2 offset = vec2(x, y) / textureSize(shadow_map, 0);
            float depth = texture(shadow_map, proj_coords.xy + offset).x;
            shadow += proj_coords.z > depth + bias ? 1.0 : 0.0;
        }
    }
    shadow /= float(pow(filter_size, 2));

    if (proj_coords.z > 1.0) {
        shadow = 0.0;
    }

    return shadow;
}

void main() {
    int sampler_idx = int(round(v_tex_idx));

    float ambient_light = 0.3;
    vec3 norm = normalize(v_normal);
    vec3 light_dir = normalize(light_pos - frag_pos);
    float diff = max(dot(norm, light_dir), 0.0);
    float light_strength = min(1.2, ambient_light + diff * (1.0 - shadow_calc(frag_pos_light)));

    vec4 textured = texture(tex_sampler[sampler_idx], v_uv).rgba * v_color;
    out_color = vec4(textured.rgb * light_strength, textured.a);
}
