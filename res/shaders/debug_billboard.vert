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


struct DebugBillboard {
	vec4 position_size;
	vec4 color;
	uint icon_kind;
	uint _pad0;
	uint _pad1;
	uint _pad2;
};

layout(buffer_reference, scalar) readonly buffer DebugBillboardBuffer {
	DebugBillboard items[];
};

layout(buffer_reference, scalar, row_major) readonly buffer SceneFrameData {
	mat4 view;
	mat4 proj;
	mat4 view_proj;
	mat4 inv_view_proj;
	vec4 camera_position;
};

layout(push_constant, scalar) uniform PushConstants {
	uint64_t scene_address;
	uint64_t data_address;
} pc;

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_color;
layout(location = 2) flat out uint out_icon_kind;


void main() {
	SceneFrameData scene = SceneFrameData(pc.scene_address);
	DebugBillboardBuffer bbuf = DebugBillboardBuffer(pc.data_address);

	uint idx = uint(gl_VertexIndex) / 6u;
	uint corner = uint(gl_VertexIndex) % 6u;
	DebugBillboard b = bbuf.items[idx];

	// CCW quad corners in world-right / world-up (front-facing in Vulkan clip space)
	const vec2 offsets[6] = vec2[6](
		vec2(-1.0, -1.0),
		vec2( 1.0, -1.0),
		vec2( 1.0,  1.0),
		vec2(-1.0, -1.0),
		vec2( 1.0,  1.0),
		vec2(-1.0,  1.0)
	);
	vec2 off = offsets[corner];

	// Camera world-space basis from row_major view matrix
	vec3 cam_right = vec3(scene.view[0][0], scene.view[1][0], scene.view[2][0]);
	vec3 cam_up = vec3(scene.view[0][1], scene.view[1][1], scene.view[2][1]);

	vec3 center = b.position_size.xyz;
	float dist = length(scene.camera_position.xyz - center);
	float half_size = clamp(dist * 0.042, 0.22, 1.35) * max(b.position_size.w, 0.1);

	vec3 world_pos = center + (cam_right * off.x + cam_up * off.y) * half_size;

	out_uv = off;
	out_color = b.color;
	out_icon_kind = b.icon_kind;
	gl_Position = scene.view_proj * vec4(world_pos, 1.0);
}
