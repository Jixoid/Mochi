#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D u_textures[];

layout(push_constant) uniform SmaaResolvePushConstants {
	vec4 rt_metrics;
	uint current_color_tex_id;
	uint previous_color_tex_id;
	uint velocity_tex_id;
	uint has_history;
} pc;

#define SMAA_GLSL_4 1
#define SMAA_PRESET_HIGH 1
#define SMAA_INCLUDE_VS 0
#define SMAA_INCLUDE_PS 1
#define SMAA_REPROJECTION 1
#define SMAA_RT_METRICS pc.rt_metrics
#include "SMAA.h"

void main() {
	vec4 current = texture(u_textures[nonuniformEXT(pc.current_color_tex_id)], in_uv);

	if (pc.has_history == 0) {
		out_color = vec4(current.rgb, 1.0);
		return;
	}

	vec2 velocity = -SMAA_DECODE_VELOCITY(texture(u_textures[nonuniformEXT(pc.velocity_tex_id)], in_uv).rg);
	vec2 prev_coord = in_uv + velocity;

	if (prev_coord.x < 0.0 || prev_coord.x > 1.0 || prev_coord.y < 0.0 || prev_coord.y > 1.0) {
		out_color = vec4(current.rgb, 1.0);
		return;
	}

	vec4 previous = texture(u_textures[nonuniformEXT(pc.previous_color_tex_id)], prev_coord);

	// Attenuate the previous pixel if the velocity is different:
	float delta = abs(current.a * current.a - previous.a * previous.a) / 5.0;
	float weight = 0.5 * clamp(1.0 - sqrt(delta) * SMAA_REPROJECTION_WEIGHT_SCALE, 0.0, 1.0);

	vec4 resolved = mix(current, previous, weight);
	out_color = vec4(resolved.rgb, 1.0);
}
