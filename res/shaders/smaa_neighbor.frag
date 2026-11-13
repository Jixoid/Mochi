#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_offset;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D u_textures[];

layout(push_constant) uniform SmaaNeighborPushConstants {
	vec4 rt_metrics;
	uint color_tex_id;
	uint blend_tex_id;
	uint _pad0;
	uint _pad1;
} pc;

#define SMAA_GLSL_4 1
#define SMAA_PRESET_HIGH 1
#define SMAA_INCLUDE_VS 0
#define SMAA_INCLUDE_PS 1
#define SMAA_RT_METRICS pc.rt_metrics
#include "SMAA.h"

void main() {
	out_color = SMAANeighborhoodBlendingPS(
		in_uv,
		in_offset,
		u_textures[nonuniformEXT(pc.color_tex_id)],
		u_textures[nonuniformEXT(pc.blend_tex_id)]
	);
}
