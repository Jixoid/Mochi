#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec2 in_pixcoord;
layout(location = 2) in vec4 in_offsets[3];
layout(location = 0) out vec4 out_weights;

layout(set = 0, binding = 0) uniform sampler2D u_textures[];

layout(push_constant) uniform SmaaBlendPushConstants {
	vec4 rt_metrics;
	uint edges_tex_id;
	uint area_tex_id;
	uint search_tex_id;
	uint _pad;
} pc;

#define SMAA_GLSL_4 1
#define SMAA_PRESET_HIGH 1
#define SMAA_INCLUDE_VS 0
#define SMAA_INCLUDE_PS 1
#define SMAA_RT_METRICS pc.rt_metrics
#include "SMAA.h"

void main() {
	out_weights = SMAABlendingWeightCalculationPS(
		in_uv,
		in_pixcoord,
		in_offsets,
		u_textures[nonuniformEXT(pc.edges_tex_id)],
		u_textures[nonuniformEXT(pc.area_tex_id)],
		u_textures[nonuniformEXT(pc.search_tex_id)],
		vec4(0.0)
	);
}
