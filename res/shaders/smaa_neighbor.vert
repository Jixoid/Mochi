#version 450

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_offset;

layout(push_constant) uniform SmaaNeighborPushConstants {
	vec4 rt_metrics;
	uint color_tex_id;
	uint blend_tex_id;
	uint _pad0;
	uint _pad1;
} pc;

#define SMAA_GLSL_4 1
#define SMAA_PRESET_HIGH 1
#define SMAA_INCLUDE_VS 1
#define SMAA_INCLUDE_PS 0
#define SMAA_RT_METRICS pc.rt_metrics
#include "SMAA.h"

void main() {
	vec2 uvs[3] = vec2[3](
		vec2(0.0, 0.0),
		vec2(0.0, 2.0),
		vec2(2.0, 0.0)
	);
	vec2 positions[3] = vec2[3](
		vec2(-1.0, -1.0),
		vec2(-1.0,  3.0),
		vec2( 3.0, -1.0)
	);
	out_uv = uvs[gl_VertexIndex];
	gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
	SMAANeighborhoodBlendingVS(out_uv, out_offset);
}
