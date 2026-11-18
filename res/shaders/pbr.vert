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

struct DirectionalLight {
	vec4 direction;
	vec4 color;
};

struct PointLight {
	vec4 position;
	vec4 color;
};

layout(buffer_reference, scalar, row_major) readonly buffer SceneFrameData {
	mat4 view;
	mat4 proj;
	mat4 view_proj;
	mat4 inv_view_proj;
	vec4 camera_position;
	vec4 ambient_color;
	uint64_t cluster_grid_address;
	uint64_t point_light_index_list_address;
	uint64_t spot_light_index_list_address;
	uint64_t point_lights_address;
	uint64_t spot_lights_address;
	vec2 cluster_screen_size;
	float z_near;
	float z_far;
	uvec4 cluster_dims;
	uint dir_light_count;
	uint point_light_count;
	uint spot_light_count;
	uint _pad0;
	uint _pad1_0;
	uint _pad1_1;
	DirectionalLight dir_lights[4];
	mat4 cascade_view_proj[4][4];
	vec4 cascade_split_depths;
	uvec4 cascade_shadow_ids[4];
	mat4 prev_view_proj;
	vec4 jitter;
};

layout(push_constant, scalar, row_major) uniform PushConstants {
	mat4 model;
	uint64_t scene_address;
	uint64_t vertex_address;
	uint64_t material_address;
	uint64_t joint_address;
	uint material_id;
	uint flags;
	uint _pad0;
	uint _pad1;
} pc;

layout(location = 0) out vec3 out_world_pos;
layout(location = 1) out vec3 out_normal;
layout(location = 2) out vec4 out_color;
layout(location = 3) out vec2 out_uv;
layout(location = 4) out vec4 out_tangent;


void main() {
	VertexBuffer vbuf = VertexBuffer(pc.vertex_address);
	SceneFrameData scene = SceneFrameData(pc.scene_address);
	Vertex v = vbuf.vertices[gl_VertexIndex];

	vec3 local_pos = v.position.xyz;
	vec3 local_normal = v.normal.xyz;
	vec3 local_tangent = v.tangent.xyz;

	if (pc.joint_address != 0 && (v.weights.x + v.weights.y + v.weights.z + v.weights.w) > 0.001) {
		JointBuffer jbuf = JointBuffer(pc.joint_address);

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
		local_normal = normalize(vec3(dot(r0.xyz, local_normal), dot(r1.xyz, local_normal), dot(r2.xyz, local_normal)));
		local_tangent = normalize(vec3(dot(r0.xyz, local_tangent), dot(r1.xyz, local_tangent), dot(r2.xyz, local_tangent)));
	}

	vec4 world_pos = pc.model * vec4(local_pos, 1.0);
	out_world_pos = world_pos.xyz;
	out_normal = normalize(mat3(pc.model) * local_normal);
	out_tangent = vec4(normalize(mat3(pc.model) * local_tangent), v.tangent.w);
	out_color = v.color;
	out_uv = v.uv0;

	gl_Position = scene.view_proj * world_pos;
}
