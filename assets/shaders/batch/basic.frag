#version 450 core

#define PI 3.141592653589
#define POINT_LIGHT_STRENGTH 300
#define MAX_DIR_LIGHT_MAPS 5
#define MAX_POINT_LIGHT_MAPS 5
#define MAX_POINT_LIGHT_COUNT 20
#define MAX_LIGHT_SRC_COUNT MAX_POINT_LIGHT_COUNT + MAX_DIR_LIGHT_MAPS
#define SHADOW_MAP_COUNT MAX_POINT_LIGHT_MAPS + MAX_DIR_LIGHT_MAPS


in vec4 v_color;
in vec2 v_uv;
in vec3 v_normal;
flat in float v_tex_idx;
in vec3 frag_pos;
in vec4 frag_pos_dir_light[MAX_DIR_LIGHT_MAPS];
in vec3 cam_position;

out vec4 out_color;

struct PointLightData {
    vec4 light_pos;
    vec4 color;
    float intensity;
    bool has_shadows;
};

struct DirLightData {
    vec4 light_pos;
    mat4 light_matrix;
    vec4 color;
    float intensity;
    vec3 direction;
};

struct LightConfig {
    vec4 color;
    float intensity;
};

layout (std140, binding = 0, column_major) uniform light_data {
    LightConfig ambient_light;
    int num_dir_lights;
    int num_point_lights;
    int num_point_light_maps;
    DirLightData dir_lights[MAX_DIR_LIGHT_MAPS];
    PointLightData point_lights[MAX_POINT_LIGHT_COUNT];
};

layout(location = 1) uniform bool transparent_pass;
layout(location = 2) uniform sampler2D shadow_samplers[MAX_DIR_LIGHT_MAPS];
layout(location = 7) uniform samplerCube cube_shadow_samplers[MAX_POINT_LIGHT_MAPS];
layout(location = 12) uniform sampler2D tex_sampler[32 - SHADOW_MAP_COUNT];

float shadow_calc(vec4 fpl, int i) {
    vec3 proj_coords = fpl.xyz / fpl.w;
    proj_coords = proj_coords * 0.5 + 0.5;

    vec3 light_dir = normalize(lights[i].light_pos.xyz - frag_pos);
    float bias = max(0.005 * (1.0 - dot(v_normal, light_dir)), 0.0001);

    int filter_size = 2;
    float shadow = 0.0;
    for (int y = -filter_size / 2; y < filter_size / 2; ++y) {
        for (int x = -filter_size / 2; x < filter_size / 2; ++x) {
            vec2 offset = vec2(x, y) / textureSize(shadow_samplers[i], 0);
            float depth = texture(shadow_samplers[i], proj_coords.xy + offset).x;
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
    vec4 textured = texture(tex_sampler[sampler_idx], v_uv).rgba * v_color;
    if (textured.a < 0.001 || (textured.a < 0.999 && !transparent_pass)) {
        discard;
    }

    vec3 final_light = vec3(0.0);
    for (int i = 0; i < num_lights; i++) {
        vec3 light_dir = normalize(lights[i].light_pos.xyz - frag_pos);
        float distance_to_light = length(frag_pos - lights[i].light_pos.xyz);
        distance_to_light = distance_to_light == 0.0 ? 0.1 : distance_to_light;
        float diff = min(max(dot(v_normal, light_dir), 0.0) / (pow(distance_to_light, 2) * 4 * PI), 1.0);
        float shadow = 1.0 - shadow_calc(frag_pos_dir_light[i], i);
        vec3 src_light = diff * shadow * lights[i].color.rgb * lights[i].intensity;
        final_light += src_light * POINT_LIGHT_STRENGTH / float(num_lights);
    }
    final_light += vec3(ambient_light.intensity);
    // clamp if final light strength is too high
    for (int i = 0; i < 3; i++) {
        final_light[i] = clamp(final_light[i], 0.0, 1.0);
    }
    // add specular lighting
    float spec_strenght = 0.3;
    for (int i = 0; i < num_lights; i++) {
        vec3 light_dir = normalize(lights[i].light_pos.xyz - frag_pos);
        vec3 view_dir = normalize(cam_position - frag_pos);
        vec3 reflect_dir = reflect(-light_dir, v_normal);
        float spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32);
        final_light += spec_strenght * spec * lights[i].color.rgb * lights[i].intensity;
    }

    out_color = vec4(textured.rgb * final_light * ambient_light.color.rgb, textured.a);
}
