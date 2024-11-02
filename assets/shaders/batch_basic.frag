#version 450 core

#define PI 3.141592653589
#define MAX_LIGHT_SRC_COUNT 5
#define POINT_LIGHT_STRENGTH 300

in vec4 v_color;
in vec2 v_uv;
in vec3 v_normal;
flat in float v_tex_idx;
in vec3 frag_pos;
in vec4 frag_pos_light[MAX_LIGHT_SRC_COUNT];
in vec3 cam_position;

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
    LightData lights[MAX_LIGHT_SRC_COUNT];
};

layout(location = 0) uniform int num_lights;
layout(location = 1) uniform sampler2D shadow_sampler[MAX_LIGHT_SRC_COUNT];
layout(location = 6) uniform sampler2D tex_sampler[32 - MAX_LIGHT_SRC_COUNT];

float shadow_calc(vec4 fpl, int i) {
    vec3 proj_coords = fpl.xyz / fpl.w;
    proj_coords = proj_coords * 0.5 + 0.5;

    vec3 light_dir = normalize(lights[i].light_pos.xyz - frag_pos);
    float bias = max(0.005 * (1.0 - dot(v_normal, light_dir)), 0.0001);

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

    vec3 final_light = vec3(0.0);
    for (int i = 0; i < num_lights; i++) {
        vec3 light_dir = normalize(lights[i].light_pos.xyz - frag_pos);
        float distance_to_light = length(frag_pos - lights[i].light_pos.xyz);
        distance_to_light = distance_to_light == 0.0 ? 0.1 : distance_to_light;

        float diff = min(max(dot(v_normal, light_dir), 0.0) / (pow(distance_to_light, 2) * 4 * PI), 1.0);
        vec3 src_light = vec3(diff * (1.0 - shadow_calc(frag_pos_light[i], i))) * lights[i].color.rgb * lights[i].intensity;
        final_light += (vec3(ambient_light.intensity) + src_light * POINT_LIGHT_STRENGTH) / float(num_lights);
    }
    final_light = num_lights > 0 ? final_light : vec3(ambient_light.intensity);
    // clamp if final light strength is too high
    for (int i = 0; i < 3; i++) {
        final_light[i] = clamp(final_light[i], 0.0, 1.0);
    }
    // add specular lighting
    float spec_strenght = 0.4;
    for (int i = 0; i < num_lights; i++) {
        vec3 light_dir = normalize(lights[i].light_pos.xyz - frag_pos);
        vec3 view_dir = normalize(cam_position - frag_pos);
        vec3 reflect_dir = reflect(-light_dir, v_normal);
        float spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32);
        final_light += spec_strenght * spec * vec3(lights[i].color) / float(num_lights);
    }

    vec4 textured = texture(tex_sampler[sampler_idx], v_uv).rgba * v_color;
    out_color = vec4(textured.rgb * final_light * ambient_light.color.rgb, textured.a);
}
