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


struct DebugVertex {
	vec4 position;
	vec4 color;
};

layout(buffer_reference, scalar) readonly buffer DebugVertexBuffer {
	DebugVertex vertices[];
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

layout(location = 0) out vec4 out_color;


void main() {
	SceneFrameData scene = SceneFrameData(pc.scene_address);
	DebugVertexBuffer vbuf = DebugVertexBuffer(pc.data_address);
	DebugVertex v = vbuf.vertices[gl_VertexIndex];

	out_color = v.color;
	gl_Position = scene.view_proj * vec4(v.position.xyz, 1.0);
}
