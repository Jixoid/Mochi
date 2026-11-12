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
};

layout(push_constant, scalar, row_major) uniform PushConstants {
	mat4 model;
	uint64_t scene_address;
	uint64_t vertex_address;
	uint64_t material_address;
	uint material_id;
	uint flags;
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

	vec4 world_pos = pc.model * vec4(v.position.xyz, 1.0);
	out_world_pos = world_pos.xyz;
	out_normal = normalize(mat3(pc.model) * v.normal.xyz);
	out_tangent = vec4(normalize(mat3(pc.model) * v.tangent.xyz), v.tangent.w);
	out_color = v.color;
	out_uv = v.uv0;

	gl_Position = scene.view_proj * world_pos;
}
