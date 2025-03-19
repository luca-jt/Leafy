#version 450 core

#define MAX_DIR_LIGHT_MAPS 5
#define MAX_POINT_LIGHT_MAPS 5
#define MAX_POINT_LIGHT_COUNT 20
#define MAX_LIGHT_SRC_COUNT MAX_POINT_LIGHT_COUNT + MAX_DIR_LIGHT_MAPS
#define SHADOW_MAP_COUNT MAX_POINT_LIGHT_MAPS + MAX_DIR_LIGHT_MAPS

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 normal;
layout (location = 5) in mat4 model; // takes up 4 attribute locations
layout (location = 9) in mat3 normal_matrix; // takes up 3 attribute locations

out vec2 v_uv;
out vec3 v_normal;
out vec3 frag_pos;
out vec4 frag_pos_dir_light[MAX_DIR_LIGHT_MAPS];
out vec3 cam_position;

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
    int num_dir_lights; // directional lights at the moment always have shadow maps
    int num_point_lights;
    int num_point_light_maps;
    DirLightData dir_lights[MAX_DIR_LIGHT_MAPS];
    PointLightData point_lights[MAX_POINT_LIGHT_COUNT];
};

layout (std140, binding = 1, column_major) uniform matrix_block {
    mat4 projection;
    mat4 view;
    vec4 cam_pos;
};

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    v_uv = uv;
    v_normal = normalize(normal_matrix * normal);
    frag_pos = vec3(model * vec4(position, 1.0));
    frag_pos_dir_light = vec4[MAX_DIR_LIGHT_MAPS](vec4(0), vec4(0), vec4(0), vec4(0), vec4(0));
    for (int i = 0; i < num_dir_lights; i++) {
        frag_pos_dir_light[i] = dir_lights[i].light_matrix * vec4(frag_pos, 1.0);
    }
    cam_position = cam_pos.xyz;
}
