/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


#version 450

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require
#extension GL_EXT_scalar_block_layout : require


struct Vertex {
	vec4 position;
	vec4 normal;
	vec4 tangent;
	vec4 color;
	vec2 uv0;
	vec2 uv1;
};

layout(buffer_reference, scalar) readonly buffer VertexBuffer {
	Vertex vertices[];
};

layout(push_constant, scalar, row_major) uniform ShadowPushConstants {
	mat4 mvp;
	uint64_t vertex_addr;
} pc;

void main() {
	VertexBuffer vb = VertexBuffer(pc.vertex_addr);
	vec4 pos = vb.vertices[gl_VertexIndex].position;
	gl_Position = pc.mvp * vec4(pos.xyz, 1.0);
}
