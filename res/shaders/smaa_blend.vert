#version 450

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec2 out_pixcoord;
layout(location = 2) out vec4 out_offsets[3];

layout(push_constant) uniform SmaaBlendPushConstants {
	vec4 rt_metrics;
	vec4 subsample_indices;
	uint edges_tex_id;
	uint area_tex_id;
	uint search_tex_id;
	uint _pad;
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
	SMAABlendingWeightCalculationVS(out_uv, out_pixcoord, out_offsets);
}
