#version 450 core

in vec3 position;
in vec4 color;
in vec2 uv;
in vec3 normal;
in float tex_idx;

out vec4 v_color;
out vec2 v_uv;
out vec3 v_normal;
out float v_tex_idx;
out vec3 frag_pos;
out vec4 frag_pos_light[5];

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

layout (std140, binding = 1, column_major) uniform matrix_block {
    mat4 projection;
    mat4 view;
};

uniform int num_lights;

void main() {
    gl_Position = projection * view * vec4(position, 1.0); // model matrix is already calculated in
    v_color = color;
    v_uv = uv;
    v_normal = normal;
    v_tex_idx = tex_idx;
    frag_pos = position;
    frag_pos_light = vec4[5](vec4(0), vec4(0), vec4(0), vec4(0), vec4(0));
    for (int i = 0; i < num_lights; i++) {
        frag_pos_light[i] = lights[i].light_matrix * vec4(frag_pos, 1.0);
    }
}
