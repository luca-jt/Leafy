#version 450 core

#define MAX_LIGHT_SRC_COUNT 5

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 normal;
layout (location = 5) in mat4 model; // takes up 4 attribute locations
layout (location = 9) in mat3 normal_matrix; // takes up 3 attribute locations

out vec2 v_uv;
out vec3 v_normal;
out vec3 frag_pos;
out vec4 frag_pos_light[MAX_LIGHT_SRC_COUNT];
out vec3 cam_position;

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

layout (std140, binding = 1, column_major) uniform matrix_block {
    mat4 projection;
    mat4 view;
    vec4 cam_pos;
};

layout(location = 0) uniform int num_lights;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    v_uv = uv;
    v_normal = normalize(normal_matrix * normal);
    frag_pos = vec3(model * vec4(position, 1.0));
    frag_pos_light = vec4[5](vec4(0), vec4(0), vec4(0), vec4(0), vec4(0));
    for (int i = 0; i < num_lights; i++) {
        frag_pos_light[i] = lights[i].light_matrix * vec4(frag_pos, 1.0);
    }
    cam_position = vec3(cam_pos);
}
