#version 450 core

#define MAX_DIR_LIGHT_MAPS 5
#define MAX_POINT_LIGHT_MAPS 5
#define MAX_POINT_LIGHT_COUNT 20
#define FAR_PLANE 100.0

in vec2 v_uv;
in vec3 v_normal;
in vec4 v_color;
in vec3 frag_pos;
in vec4 frag_pos_dir_light[MAX_DIR_LIGHT_MAPS];
in vec3 cam_position;
in mat3 TBN;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 bright_color;

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
    DirLightData dir_lights[MAX_DIR_LIGHT_MAPS];
    PointLightData point_lights[MAX_POINT_LIGHT_COUNT];
    int num_dir_lights;
    int num_point_lights;
};

layout(location = 0) uniform vec4 color; // object color
layout(location = 1) uniform sampler2D tex_sampler;
layout(location = 2) uniform bool transparent_pass;
layout(location = 3) uniform sampler2D shadow_samplers[MAX_DIR_LIGHT_MAPS];
layout(location = 8) uniform samplerCube cube_shadow_samplers[MAX_POINT_LIGHT_MAPS];
layout(location = 13) uniform vec3 ambient_color;
layout(location = 14) uniform vec3 diffuse_color;
layout(location = 15) uniform vec3 specular_color;
layout(location = 16) uniform float shininess;
layout(location = 17) uniform sampler2D ambient_sampler;
layout(location = 18) uniform sampler2D diffuse_sampler;
layout(location = 19) uniform sampler2D specular_sampler;
layout(location = 20) uniform sampler2D normal_map_sampler;
layout(location = 21) uniform bool use_normal_map;

vec3 sample_offset_directions[20] = vec3[]
(
   vec3( 1,  1,  1), vec3( 1, -1,  1), vec3(-1, -1,  1), vec3(-1,  1,  1),
   vec3( 1,  1, -1), vec3( 1, -1, -1), vec3(-1, -1, -1), vec3(-1,  1, -1),
   vec3( 1,  1,  0), vec3( 1, -1,  0), vec3(-1, -1,  0), vec3(-1,  1,  0),
   vec3( 1,  0,  1), vec3(-1,  0,  1), vec3( 1,  0, -1), vec3(-1,  0, -1),
   vec3( 0,  1,  1), vec3( 0, -1,  1), vec3( 0, -1, -1), vec3( 0,  1, -1)
);

float shadow_calc_point(int i, int shadow_map_index) {
    vec3 frag_to_light = frag_pos - point_lights[i].light_pos.xyz;
    float current_depth = length(frag_to_light);
    float bias = max(0.25 * (1.0 - dot(v_normal, normalize(-frag_to_light))), 0.15);
    float disk_radius = (1.0 + (length(cam_position - frag_pos) / FAR_PLANE)) / 200.0;

    int samples = 20;
    float offset  = 0.1;
    float shadow = 0.0;
    for (int j = 0; j < samples; ++j) {
        float depth = texture(cube_shadow_samplers[shadow_map_index], frag_to_light + sample_offset_directions[j] * disk_radius).r * FAR_PLANE;
        shadow += current_depth > depth + bias ? 1.0 : 0.0;
    }
    shadow /= float(samples);

    return shadow;
}

float shadow_calc_dir(int i) {
    vec3 proj_coords = frag_pos_dir_light[i].xyz / frag_pos_dir_light[i].w;
    proj_coords = proj_coords * 0.5 + 0.5;

    float bias = max(0.05 * (1.0 - dot(v_normal, -dir_lights[i].direction)), 0.001);
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
    vec4 textured = texture(tex_sampler, v_uv).rgba * color * v_color;
    if (textured.a < 0.001 || (textured.a < 0.999 && !transparent_pass)) {
        discard;
    }

    vec3 frag_normal;
    if (use_normal_map) {
        frag_normal = texture(normal_map_sampler, v_uv).rgb * 2.0 - 1.0;
        frag_normal = normalize(TBN * frag_normal);
    } else {
        frag_normal = v_normal;
    }

    vec3 material_ambient_color = ambient_color * texture(ambient_sampler, v_uv).rgb;
    vec3 material_diffuse_color = diffuse_color * texture(diffuse_sampler, v_uv).rgb;
    vec3 material_specular_color = specular_color * texture(specular_sampler, v_uv).rgb;

    vec3 final_light = material_ambient_color * ambient_light.color.rgb * ambient_light.intensity;
    // directional lights
    for (int i = 0; i < num_dir_lights; i++) {
        float diff = min(max(dot(frag_normal, -dir_lights[i].direction), 0.0), 1.0);
        float shadow = 1.0 - shadow_calc_dir(i) / float(num_dir_lights + num_point_lights);
        float distance_to_light = length(frag_pos - dir_lights[i].light_pos.xyz);
        distance_to_light = distance_to_light == 0.0 ? 0.1 : distance_to_light;
        float attenuation = 1.0 / distance_to_light;
        vec3 src_light = diff * attenuation * shadow * dir_lights[i].color.rgb * material_diffuse_color * dir_lights[i].intensity;
        final_light += src_light;
    }
    // point lights
    int point_light_map_index = 0;
    for (int i = 0; i < num_point_lights; i++) {
        vec3 light_dir = normalize(point_lights[i].light_pos.xyz - frag_pos);
        float diff = min(max(dot(frag_normal, light_dir), 0.0), 1.0);
        float shadow = 1.0;
        if (point_lights[i].has_shadows) {
            shadow -= shadow_calc_point(i, point_light_map_index) / float(num_dir_lights + num_point_lights);
            point_light_map_index += 1;
        }
        float distance_to_light = length(frag_pos - point_lights[i].light_pos.xyz);
        distance_to_light = distance_to_light == 0.0 ? 0.1 : distance_to_light;
        float attenuation = 1.0 / distance_to_light;
        vec3 src_light = diff * attenuation * shadow * point_lights[i].color.rgb * material_diffuse_color * point_lights[i].intensity;
        final_light += src_light;
    }
    // add specular lighting
    float spec_strenght = 0.3;
    for (int i = 0; i < num_dir_lights; i++) {
        vec3 view_dir = normalize(cam_position - frag_pos);
        vec3 halfway_dir = normalize(-dir_lights[i].direction + view_dir);
        float spec = pow(max(dot(frag_normal, halfway_dir), 0.0), shininess);
        final_light += spec_strenght * spec * dir_lights[i].color.rgb * material_specular_color * dir_lights[i].intensity;
    }
    for (int i = 0; i < num_point_lights; i++) {
        vec3 light_dir = normalize(point_lights[i].light_pos.xyz - frag_pos);
        vec3 view_dir = normalize(cam_position - frag_pos);
        vec3 halfway_dir = normalize(light_dir + view_dir);
        float spec = pow(max(dot(frag_normal, halfway_dir), 0.0), shininess);
        final_light += spec_strenght * spec * point_lights[i].color.rgb * material_specular_color * point_lights[i].intensity;
    }

    out_color = vec4(textured.rgb * final_light, textured.a);
    bright_color = dot(out_color.rgb, vec3(0.2126, 0.7152, 0.0722)) > 1.0 ? vec4(out_color.rgb, 1.0) : vec4(0.0, 0.0, 0.0, 1.0);
}
