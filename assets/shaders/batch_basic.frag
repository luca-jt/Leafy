#version 450 core

in vec4 v_color;
in vec2 v_uv;
in vec3 v_normal;
in float v_tex_idx;
in vec3 frag_pos;
in vec4 frag_pos_light[5];

out vec4 out_color;

struct LightData {
    vec4 light_pos;
    mat4 light_matrix;
    vec4 color;
    float intensity;
};

struct LightConfig {
    vec4 color;
    float intensity;
};

layout (std140, binding = 0, column_major) uniform light_data {
    LightConfig ambient_light;
    LightData lights[5];
};

uniform sampler2D shadow_sampler[5];
uniform sampler2D tex_sampler[27];
uniform int num_lights;

float shadow_calc(vec4 fpl, int i) {
    vec3 proj_coords = fpl.xyz / fpl.w;
    proj_coords = proj_coords * 0.5 + 0.5;

    vec3 light_dir = normalize(lights[i].light_pos.xyz - frag_pos);
    float bias = max(0.005 * (1.0 - dot(normalize(v_normal), light_dir)), 0.0002);

    int filter_size = 2;
    float shadow = 0.0;
    for (int y = -filter_size / 2; y < filter_size / 2; ++y) {
        for (int x = -filter_size / 2; x < filter_size / 2; ++x) {
            vec2 offset = vec2(x, y) / textureSize(shadow_sampler[i], 0);
            float depth = texture(shadow_sampler[i], proj_coords.xy + offset).x;
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
    vec3 norm = normalize(v_normal);

    vec3 final_light = vec3(0.0);
    for (int i = 0; i < num_lights; i++) {
        vec3 light_dir = normalize(lights[i].light_pos.xyz - frag_pos);
        float diff = max(dot(norm, light_dir), 0.0);
        vec3 src_light = vec3(diff * (1.0 - shadow_calc(frag_pos_light[i], i))) * lights[i].color.rgb * lights[i].intensity;
        final_light += (vec3(ambient_light.intensity) + src_light) / float(num_lights);
    }
    final_light = num_lights > 0 ? final_light : vec3(ambient_light.intensity);

    vec4 textured = texture(tex_sampler[sampler_idx], v_uv).rgba * v_color;
    out_color = vec4(textured.rgb * final_light * ambient_light.color.rgb, textured.a);
}
