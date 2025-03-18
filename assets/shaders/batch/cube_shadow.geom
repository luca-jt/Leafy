#version 450 core

layout (triangles) in;
layout (triangle_strip, max_vertices=18) out;

out vec4 frag_pos;

layout(location = 33) uniform mat4 base_light_matrix; // this is the base matrix that generates the other ones

void main() {
    for (int face = 0; face < 6; ++face) {
        gl_Layer = face;

        for (int i = 0; i < 3; ++i) {
            frag_pos = gl_in[i].gl_Position;
            gl_Position = shadowMatrices[face] * frag_pos;
            EmitVertex();
        }

        EndPrimitive();
    }
}
