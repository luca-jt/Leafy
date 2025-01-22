#version 450 core

#define PI 3.141592653589
#define MAX_LIGHT_SRC_COUNT 5
#define POINT_LIGHT_STRENGTH 300

in vec2 v_uv;
in vec3 v_normal;
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
layout(location = 1) uniform vec4 color;
layout(location = 2) uniform sampler2D tex_sampler;
layout(location = 3) uniform bool transparent_pass;
layout(location = 4) uniform sampler2D shadow_sampler[MAX_LIGHT_SRC_COUNT];

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
    vec4 textured = texture(tex_sampler, v_uv).rgba * color;
    if (textured.a < 0.001 || (textured.a < 0.999 && !transparent_pass)) {
        discard;
    }

    vec3 final_light = vec3(0.0);
    for (int i = 0; i < num_lights; i++) {
        vec3 light_dir = normalize(lights[i].light_pos.xyz - frag_pos);
        float distance_to_light = length(frag_pos - lights[i].light_pos.xyz);
        distance_to_light = distance_to_light == 0.0 ? 0.1 : distance_to_light;

        float diff = min(max(dot(v_normal, light_dir), 0.0) / (pow(distance_to_light, 2) * 4 * PI), 1.0);
        float shadow = 1.0 - shadow_calc(frag_pos_light[i], i);
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
