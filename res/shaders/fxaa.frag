#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D u_textures[];

layout(push_constant) uniform FxaaPushConstants {
	uint texture_id;
	float texel_w;
	float texel_h;
	float _pad;
} pc;

float rgb_to_luma(vec3 rgb) {
	return dot(rgb, vec3(0.299, 0.587, 0.114));
}

void main() {
	vec2 texel = vec2(pc.texel_w, pc.texel_h);

	vec3 rgbM = texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv).rgb;
	float lumaM = rgb_to_luma(rgbM);

	float lumaNW = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2(-texel.x, -texel.y)).rgb);
	float lumaNE = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2( texel.x, -texel.y)).rgb);
	float lumaSW = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2(-texel.x,  texel.y)).rgb);
	float lumaSE = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2( texel.x,  texel.y)).rgb);

	float lumaM_N = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2(0.0, -texel.y)).rgb);
	float lumaM_S = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2(0.0,  texel.y)).rgb);
	float lumaM_W = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2(-texel.x, 0.0)).rgb);
	float lumaM_E = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2( texel.x, 0.0)).rgb);

	float lumaMin = min(lumaM, min(min(lumaM_N, lumaM_W), min(lumaM_S, lumaM_E)));
	float lumaMax = max(lumaM, max(max(lumaM_N, lumaM_W), max(lumaM_S, lumaM_E)));
	float range = lumaMax - lumaMin;

	// Early exit on low contrast edges
	if (range < max(0.05, lumaMax * 0.125)) {
		out_color = vec4(rgbM, 1.0);
		return;
	}

	// Sub-pixel antialiasing
	float lumaL = (lumaM_N + lumaM_S + lumaM_W + lumaM_E) * 0.25;
	float rangeL = abs(lumaL - lumaM);
	float blendL = max(0.0, (rangeL / range) - 0.25) * 1.3333;
	blendL = min(blendL, 0.75);

	// Edge direction test
	float edgeVert = abs((0.25 * lumaNW) + (-0.5 * lumaM_N) + (0.25 * lumaNE)) +
	                 abs((0.50 * lumaM_W) + (-1.0 * lumaM  ) + (0.50 * lumaM_E)) +
	                 abs((0.25 * lumaSW) + (-0.5 * lumaM_S) + (0.25 * lumaSE));

	float edgeHorz = abs((0.25 * lumaNW) + (-0.5 * lumaM_W) + (0.25 * lumaSW)) +
	                 abs((0.50 * lumaM_N) + (-1.0 * lumaM  ) + (0.50 * lumaM_S)) +
	                 abs((0.25 * lumaNE) + (-0.5 * lumaM_E) + (0.25 * lumaSE));

	bool isHorizontal = (edgeHorz >= edgeVert);

	float stepLength = isHorizontal ? texel.y : texel.x;
	float luma1 = isHorizontal ? lumaM_N : lumaM_W;
	float luma2 = isHorizontal ? lumaM_S : lumaM_E;
	float gradient1 = abs(luma1 - lumaM);
	float gradient2 = abs(luma2 - lumaM);

	bool is1Steeper = gradient1 >= gradient2;
	float gradientScaled = 0.25 * max(gradient1, gradient2);

	vec2 currentUv = in_uv;
	if (isHorizontal) {
		currentUv.y += (is1Steeper ? -stepLength : stepLength) * 0.5;
	} else {
		currentUv.x += (is1Steeper ? -stepLength : stepLength) * 0.5;
	}

	vec2 offset = isHorizontal ? vec2(texel.x, 0.0) : vec2(0.0, texel.y);
	vec2 uv1 = currentUv - offset;
	vec2 uv2 = currentUv + offset;

	float lumaEnd1 = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], uv1).rgb) - lumaM;
	float lumaEnd2 = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], uv2).rgb) - lumaM;

	bool reached1 = abs(lumaEnd1) >= gradientScaled;
	bool reached2 = abs(lumaEnd2) >= gradientScaled;
	bool reachedBoth = reached1 && reached2;

	if (!reached1) { uv1 -= offset; }
	if (!reached2) { uv2 += offset; }

	if (!reachedBoth) {
		for (int i = 2; i < 8; i++) {
			if (!reached1) {
				lumaEnd1 = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], uv1).rgb) - lumaM;
				reached1 = abs(lumaEnd1) >= gradientScaled;
				if (!reached1) { uv1 -= offset; }
			}
			if (!reached2) {
				lumaEnd2 = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], uv2).rgb) - lumaM;
				reached2 = abs(lumaEnd2) >= gradientScaled;
				if (!reached2) { uv2 += offset; }
			}
			if (reached1 && reached2) { break; }
		}
	}

	float dist1 = isHorizontal ? (in_uv.x - uv1.x) : (in_uv.y - uv1.y);
	float dist2 = isHorizontal ? (uv2.x - in_uv.x) : (uv2.y - in_uv.y);

	bool isDirection1 = dist1 < dist2;
	float distMin = min(dist1, dist2);
	float edgeThickness = dist1 + dist2;

	float pixelOffset = -distMin / edgeThickness + 0.5;
	bool isLumaCenterSmaller = lumaM < lumaL;
	bool correctVariation = ((isDirection1 ? lumaEnd1 : lumaEnd2) < 0.0) != isLumaCenterSmaller;
	float finalOffset = correctVariation ? pixelOffset : 0.0;

	float finalBlend = max(finalOffset, blendL);

	vec2 finalUv = in_uv;
	if (isHorizontal) {
		finalUv.y += finalBlend * (is1Steeper ? -stepLength : stepLength);
	} else {
		finalUv.x += finalBlend * (is1Steeper ? -stepLength : stepLength);
	}

	out_color = vec4(texture(u_textures[nonuniformEXT(pc.texture_id)], finalUv).rgb, 1.0);
}
