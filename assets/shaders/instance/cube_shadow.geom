#version 450 core

layout(triangles) in;
layout(triangle_strip, max_vertices = 18) out;

in vec2 v_uv;

out vec4 frag_pos;
out vec2 g_uv;

layout(location = 33) uniform mat4 light_matrices[6];

void main() {
    g_uv = v_uv;

    for (int face = 0; face < 6; ++face) {
        gl_Layer = face;

        for (int i = 0; i < 3; ++i) {
            frag_pos = gl_in[i].gl_Position;
            gl_Position = light_matrices[face] * frag_pos;
            EmitVertex();
        }

        EndPrimitive();
    }
}
