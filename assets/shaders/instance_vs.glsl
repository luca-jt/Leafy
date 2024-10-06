#version 450 core

in vec3 position;
in vec2 uv;
in vec3 normal;
layout (location = 5) in mat4 model; // takes up 4 attribute locations

out vec4 v_color;
out vec2 v_uv;
out vec3 v_normal;
out vec3 frag_pos;
out vec4 frag_pos_light[5];

struct LightData {
    vec4 light_pos;
    mat4 light_matrix;
};

struct LightConfig {
    vec4 color;
    float intensity;
};

layout (std140, binding = 0, column_major) uniform light_data {
    LightConfig ambient_light;
    LightData lights[5];
};

uniform vec4 color;
uniform mat4 projection;
uniform mat4 view;
uniform int num_lights;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    v_color = color;
    v_uv = uv;
    v_normal = normal;
    frag_pos = vec3(model * vec4(position, 1.0));
    frag_pos_light = vec4[5](vec4(0), vec4(0), vec4(0), vec4(0), vec4(0));
    for (int i = 0; i < num_lights; i++) {
        frag_pos_light[i] = lights[i].light_matrix * vec4(frag_pos, 1.0);
    }
}
