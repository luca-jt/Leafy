#version 450 core

#define MAX_DIR_LIGHT_MAPS 5
#define MAX_POINT_LIGHT_MAPS 5
#define MAX_POINT_LIGHT_COUNT 20
#define MAX_LIGHT_SRC_COUNT MAX_POINT_LIGHT_COUNT + MAX_DIR_LIGHT_MAPS
#define SHADOW_MAP_COUNT MAX_POINT_LIGHT_MAPS + MAX_DIR_LIGHT_MAPS

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec2 uv;
layout(location = 3) in vec3 normal;
layout(location = 4) in float tex_idx;

out vec4 v_color;
out vec2 v_uv;
out vec3 v_normal;
flat out float v_tex_idx;
out vec3 frag_pos;
out vec4 frag_pos_light[MAX_LIGHT_SRC_COUNT];
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
    int num_dir_lights;
    int num_point_lights;
    int num_point_light_maps;
    DirLightData dir_lights[MAX_DIR_LIGHT_MAPS];
    PointLightData point_lights[MAX_POINT_LIGHT_COUNT];
    mat4 point_light_matrices[MAX_POINT_LIGHT_MAPS];
};

layout (std140, binding = 1, column_major) uniform matrix_block {
    mat4 projection;
    mat4 view;
    vec4 cam_pos;
};

void main() {
    gl_Position = projection * view * vec4(position, 1.0); // model matrix is already calculated in
    v_color = color;
    v_uv = uv;
    v_normal = normalize(normal);
    v_tex_idx = tex_idx;
    frag_pos = position;
    frag_pos_light = vec4[5](vec4(0), vec4(0), vec4(0), vec4(0), vec4(0));
    for (int i = 0; i < num_lights; i++) {
        frag_pos_light[i] = lights[i].light_matrix * vec4(frag_pos, 1.0);
    }
    cam_position = vec3(cam_pos);
}
