#version 450 core

layout(triangles) in;
layout(triangle_strip, max_vertices = 18) out;

in vec4 v_color[];
in vec2 v_uv[];
flat in float v_tex_idx[];

out vec4 frag_pos;
out vec4 g_color;
out vec2 g_uv;
flat out float g_tex_idx;

layout(location = 0) uniform mat4 light_matrices[6];

void main() {
    for (int face = 0; face < 6; ++face) {
        gl_Layer = face;

        for (int i = 0; i < 3; ++i) {
            frag_pos = gl_in[i].gl_Position;
            g_color = v_color[i];
            g_uv = v_uv[i];
            g_tex_idx = v_tex_idx[i];
            gl_Position = light_matrices[face] * frag_pos;
            EmitVertex();
        }

        EndPrimitive();
    }
}
