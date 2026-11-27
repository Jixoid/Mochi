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

layout(location = 0) in vec2 in_ndc;
layout(location = 0) out vec4 out_color;


float grid_line(vec2 coord, float scale) {
	vec2 p = coord / scale;
	vec2 deriv = fwidth(p);
	vec2 grid = abs(fract(p - 0.5) - 0.5) / max(deriv, vec2(1e-4));
	float line = min(grid.x, grid.y);
	return 1.0 - min(line, 1.0);
}


void main() {
	SceneFrameData scene = SceneFrameData(pc.scene_address);

	vec4 near_pt = scene.inv_view_proj * vec4(in_ndc, 0.0, 1.0);
	near_pt /= near_pt.w;
	vec4 far_pt = scene.inv_view_proj * vec4(in_ndc, 1.0, 1.0);
	far_pt /= far_pt.w;

	float denom = far_pt.y - near_pt.y;
	if (abs(denom) < 1e-6) {
		discard;
	}

	float t = -near_pt.y / denom;
	if (t <= 0.0) {
		discard;
	}

	vec3 world_pos = near_pt.xyz + t * (far_pt.xyz - near_pt.xyz);
	vec4 clip_pos = scene.view_proj * vec4(world_pos, 1.0);
	float depth = clip_pos.z / clip_pos.w;
	if (depth < 0.0 || depth > 1.0) {
		discard;
	}
	gl_FragDepth = depth;

	float dist = length(world_pos.xz - scene.camera_position.xz);
	float fade = 1.0 - smoothstep(18.0, 90.0, dist);
	if (fade <= 0.002) {
		discard;
	}

	float minor = grid_line(world_pos.xz, 1.0);
	float major = grid_line(world_pos.xz, 10.0);

	vec2 fw = max(fwidth(world_pos.xz), vec2(1e-4));
	float x_axis = 1.0 - min(abs(world_pos.z) / (fw.y * 1.4), 1.0);
	float z_axis = 1.0 - min(abs(world_pos.x) / (fw.x * 1.4), 1.0);

	vec3 col = vec3(0.32, 0.35, 0.42);
	float alpha = minor * 0.24;

	if (major > 0.01) {
		col = mix(col, vec3(0.48, 0.52, 0.60), major);
		alpha = max(alpha, major * 0.48);
	}
	if (x_axis > 0.01) {
		col = mix(col, vec3(0.88, 0.32, 0.34), x_axis);
		alpha = max(alpha, x_axis * 0.85);
	}
	if (z_axis > 0.01) {
		col = mix(col, vec3(0.30, 0.56, 0.92), z_axis);
		alpha = max(alpha, z_axis * 0.85);
	}

	alpha *= fade;
	if (alpha < 0.005) {
		discard;
	}

	out_color = vec4(col, alpha);
}
