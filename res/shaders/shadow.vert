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
	uvec4 joints;
	vec4 weights;
};

struct JointTransform {
	vec4 row0;
	vec4 row1;
	vec4 row2;
};

layout(buffer_reference, scalar) readonly buffer VertexBuffer {
	Vertex vertices[];
};

layout(buffer_reference, scalar) readonly buffer JointBuffer {
	JointTransform joints[];
};

layout(push_constant, scalar, row_major) uniform ShadowPushConstants {
	mat4 mvp;
	uint64_t vertex_addr;
	uint64_t joint_addr;
} pc;

void main() {
	VertexBuffer vb = VertexBuffer(pc.vertex_addr);
	Vertex v = vb.vertices[gl_VertexIndex];

	vec3 local_pos = v.position.xyz;

	if (pc.joint_addr != 0 && (v.weights.x + v.weights.y + v.weights.z + v.weights.w) > 0.001) {
		JointBuffer jbuf = JointBuffer(pc.joint_addr);

		vec4 r0 = v.weights.x * jbuf.joints[v.joints.x].row0 +
		          v.weights.y * jbuf.joints[v.joints.y].row0 +
		          v.weights.z * jbuf.joints[v.joints.z].row0 +
		          v.weights.w * jbuf.joints[v.joints.w].row0;

		vec4 r1 = v.weights.x * jbuf.joints[v.joints.x].row1 +
		          v.weights.y * jbuf.joints[v.joints.y].row1 +
		          v.weights.z * jbuf.joints[v.joints.z].row1 +
		          v.weights.w * jbuf.joints[v.joints.w].row1;

		vec4 r2 = v.weights.x * jbuf.joints[v.joints.x].row2 +
		          v.weights.y * jbuf.joints[v.joints.y].row2 +
		          v.weights.z * jbuf.joints[v.joints.z].row2 +
		          v.weights.w * jbuf.joints[v.joints.w].row2;

		vec4 pos_h = vec4(local_pos, 1.0);
		local_pos = vec3(dot(r0, pos_h), dot(r1, pos_h), dot(r2, pos_h));
	}

	gl_Position = pc.mvp * vec4(local_pos, 1.0);
}
