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

// NVIDIA FXAA 3.11 Quality Tuning Constants:
// FXAA_EDGE_THRESHOLD: Edge detection sensitivity (0.166 gives crisp, sharp edges without blurring textures)
#define FXAA_EDGE_THRESHOLD      0.166
// FXAA_EDGE_THRESHOLD_MIN: Minimum contrast in dark regions
#define FXAA_EDGE_THRESHOLD_MIN  0.0625
// FXAA_SUBPIX: Controls sub-pixel aliasing removal.
// 0.00 gives native pixel sharpness (zero texture blurring, only geometric silhouette edges are smoothed).
#define FXAA_SUBPIX              0.00

float rgb_to_luma(vec3 rgb) {
	return dot(rgb, vec3(0.299, 0.587, 0.114));
}

void main() {
	vec2 texel = vec2(pc.texel_w, pc.texel_h);

	// 1. Sample center pixel
	vec3 rgbM = texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv).rgb;
	float lumaM = rgb_to_luma(rgbM);

	// 2. Sample 4 cross neighbors (North, South, West, East)
	float lumaN = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2(0.0, -texel.y)).rgb);
	float lumaS = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2(0.0,  texel.y)).rgb);
	float lumaW = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2(-texel.x, 0.0)).rgb);
	float lumaE = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2( texel.x, 0.0)).rgb);

	// 3. Early exit on flat / low-contrast regions (skips corner and edge sampling on ~90% of screen pixels)
	float lumaMin = min(lumaM, min(min(lumaN, lumaW), min(lumaS, lumaE)));
	float lumaMax = max(lumaM, max(max(lumaN, lumaW), max(lumaS, lumaE)));
	float range = lumaMax - lumaMin;

	if (range < max(FXAA_EDGE_THRESHOLD_MIN, lumaMax * FXAA_EDGE_THRESHOLD)) {
		out_color = vec4(rgbM, 1.0);
		return;
	}

	// 4. Sample corners only on detected high-contrast edges
	float lumaNW = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2(-texel.x, -texel.y)).rgb);
	float lumaNE = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2( texel.x, -texel.y)).rgb);
	float lumaSW = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2(-texel.x,  texel.y)).rgb);
	float lumaSE = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2( texel.x,  texel.y)).rgb);

	// 5. Sub-pixel antialiasing test (smoothstep S-curve prevents texture blurring)
	float lumaL = (lumaN + lumaS + lumaE + lumaW) * 2.0 + (lumaNW + lumaNE + lumaSW + lumaSE);
	float subpixA = abs(lumaL * (1.0 / 12.0) - lumaM);
	float subpixB = clamp(subpixA / range, 0.0, 1.0);
	float subpixC = (-2.0 * subpixB + 3.0) * subpixB * subpixB;
	float subpixBlend = subpixC * subpixC * FXAA_SUBPIX;

	// 6. Edge orientation test
	// Horizontal edge separates top & bottom (gradient across Y)
	float edgeHorz = abs((lumaNW - 2.0 * lumaW + lumaSW) * 0.25) +
	                 abs((lumaN  - 2.0 * lumaM + lumaS ) * 0.50) +
	                 abs((lumaNE - 2.0 * lumaE + lumaSE) * 0.25);

	// Vertical edge separates left & right (gradient across X)
	float edgeVert = abs((lumaNW - 2.0 * lumaN + lumaNE) * 0.25) +
	                 abs((lumaW  - 2.0 * lumaM + lumaE ) * 0.50) +
	                 abs((lumaSW - 2.0 * lumaS + lumaSE) * 0.25);

	bool isHorizontal = (edgeHorz >= edgeVert);

	// 7. Gradient along edge normal
	float lumaPos = isHorizontal ? lumaN : lumaW;
	float lumaNeg = isHorizontal ? lumaS : lumaE;
	float gradPos = abs(lumaPos - lumaM);
	float gradNeg = abs(lumaNeg - lumaM);

	bool isPosSteeper = (gradPos >= gradNeg);
	float stepLength = isHorizontal ? texel.y : texel.x;
	if (isPosSteeper) {
		stepLength = -stepLength;
	}

	float lumaOpposite = isPosSteeper ? lumaPos : lumaNeg;
	float gradientScaled = max(gradPos, gradNeg) * 0.25;

	// 8. Search along edge direction in stepped intervals
	vec2 currentUv = in_uv;
	if (isHorizontal) {
		currentUv.y += stepLength * 0.5;
	} else {
		currentUv.x += stepLength * 0.5;
	}

	vec2 offset = isHorizontal ? vec2(texel.x, 0.0) : vec2(0.0, texel.y);
	vec2 uv1 = currentUv - offset;
	vec2 uv2 = currentUv + offset;

	float lumaEdgeCenter = (lumaM + lumaOpposite) * 0.5;

	float lumaEnd1 = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], uv1).rgb) - lumaEdgeCenter;
	float lumaEnd2 = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], uv2).rgb) - lumaEdgeCenter;

	bool reached1 = abs(lumaEnd1) >= gradientScaled;
	bool reached2 = abs(lumaEnd2) >= gradientScaled;

	const float steps[4] = float[4](1.5, 2.0, 2.0, 4.0);

	for (int i = 0; i < 4; i++) {
		if (!reached1) {
			uv1 -= offset * steps[i];
			lumaEnd1 = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], uv1).rgb) - lumaEdgeCenter;
			reached1 = abs(lumaEnd1) >= gradientScaled;
		}
		if (!reached2) {
			uv2 += offset * steps[i];
			lumaEnd2 = rgb_to_luma(texture(u_textures[nonuniformEXT(pc.texture_id)], uv2).rgb) - lumaEdgeCenter;
			reached2 = abs(lumaEnd2) >= gradientScaled;
		}
		if (reached1 && reached2) {
			break;
		}
	}

	// 9. Compute distance to ends along edge
	float dist1 = isHorizontal ? (in_uv.x - uv1.x) : (in_uv.y - uv1.y);
	float dist2 = isHorizontal ? (uv2.x - in_uv.x) : (uv2.y - in_uv.y);

	bool isDirection1 = (dist1 < dist2);
	float distMin = min(dist1, dist2);
	float edgeThickness = dist1 + dist2;

	float pixelOffset = -distMin / edgeThickness + 0.5;

	bool isLumaCenterSmaller = lumaM < lumaEdgeCenter;
	bool correctVariation = ((isDirection1 ? lumaEnd1 : lumaEnd2) < 0.0) != isLumaCenterSmaller;
	float finalOffset = correctVariation ? pixelOffset : 0.0;

	// 10. Blend edge offset with subpixel antialiasing
	float finalBlend = max(finalOffset, subpixBlend);

	vec2 finalUv = in_uv;
	if (isHorizontal) {
		finalUv.y += finalBlend * stepLength;
	} else {
		finalUv.x += finalBlend * stepLength;
	}

	out_color = vec4(texture(u_textures[nonuniformEXT(pc.texture_id)], finalUv).rgb, 1.0);
}
