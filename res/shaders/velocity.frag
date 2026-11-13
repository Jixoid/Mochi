#version 450
#extension GL_EXT_nonuniform_qualifier : enable
#extension GL_EXT_buffer_reference : require
#extension GL_EXT_scalar_block_layout : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec2 out_velocity;

layout(set = 0, binding = 0) uniform sampler2D u_textures[];

struct DirectionalLight {
	vec4 direction;
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

layout(push_constant) uniform VelocityPushConstants {
	uint64_t scene_addr;
	uint depth_tex_id;
	uint _pad;
} pc;

void main() {
	float depth = texture(u_textures[nonuniformEXT(pc.depth_tex_id)], in_uv).r;

	if (depth >= 1.0) {
		out_velocity = vec2(0.0);
		return;
	}

	SceneFrameData scene = SceneFrameData(pc.scene_addr);

	vec2 ndc_xy = in_uv * 2.0 - 1.0;
	vec4 clip_pos = vec4(ndc_xy, depth, 1.0);
	vec4 world_pos = scene.inv_view_proj * clip_pos;
	world_pos /= world_pos.w;

	vec4 prev_clip = scene.prev_view_proj * world_pos;
	vec2 prev_ndc = prev_clip.xy / prev_clip.w;

	vec2 velocity = (ndc_xy - prev_ndc) * 0.5;

	out_velocity = velocity;
}
