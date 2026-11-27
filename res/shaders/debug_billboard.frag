/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


#version 450

layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_color;
layout(location = 2) flat in uint in_icon_kind;

layout(location = 0) out vec4 out_color;


float sd_box(vec2 p, vec2 b) {
	vec2 d = abs(p) - b;
	return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);
}

float sd_segment(vec2 p, vec2 a, vec2 b) {
	vec2 pa = p - a;
	vec2 ba = b - a;
	float h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-6), 0.0, 1.0);
	return length(pa - ba * h);
}

float sd_camera_icon(vec2 p) {
	float body = sd_box(p - vec2(-0.14, 0.0), vec2(0.30, 0.22)) - 0.04;
	float lens_top = sd_segment(p, vec2(0.18, 0.08), vec2(0.42, 0.22)) - 0.055;
	float lens_bot = sd_segment(p, vec2(0.18, -0.08), vec2(0.42, -0.22)) - 0.055;
	float lens_end = sd_segment(p, vec2(0.42, -0.22), vec2(0.42, 0.22)) - 0.055;
	float lens_fill = sd_box(p - vec2(0.30, 0.0), vec2(0.11, 0.11));
	float lens = min(min(lens_top, lens_bot), min(lens_end, lens_fill));
	return min(body, lens);
}

float sd_directional_light_icon(vec2 p) {
	float core = length(p) - 0.20;
	float rays = 1e5;
	for (int i = 0; i < 8; i++) {
		float a = float(i) * 0.78539816;
		vec2 dir = vec2(cos(a), sin(a));
		rays = min(rays, sd_segment(p, dir * 0.32, dir * 0.50) - 0.042);
	}
	return min(core, rays);
}

float sd_point_light_icon(vec2 p) {
	float bulb = length(p - vec2(0.0, 0.08)) - 0.24;
	float neck = sd_box(p - vec2(0.0, -0.22), vec2(0.12, 0.09)) - 0.02;
	float base = sd_segment(p, vec2(-0.10, -0.37), vec2(0.10, -0.37)) - 0.035;
	float ring = abs(length(p - vec2(0.0, 0.08)) - 0.40) - 0.028;
	return min(min(bulb, neck), min(base, ring));
}

float sd_spot_light_icon(vec2 p) {
	float head = length(p - vec2(0.0, 0.28)) - 0.14;
	float left_edge = sd_segment(p, vec2(-0.08, 0.22), vec2(-0.36, -0.34)) - 0.042;
	float right_edge = sd_segment(p, vec2(0.08, 0.22), vec2(0.36, -0.34)) - 0.042;
	float base_arc = sd_segment(p, vec2(-0.34, -0.34), vec2(0.34, -0.34)) - 0.038;
	float inner_ray = sd_segment(p, vec2(0.0, 0.16), vec2(0.0, -0.24)) - 0.026;
	return min(min(head, inner_ray), min(min(left_edge, right_edge), base_arc));
}


void main() {
	float r = length(in_uv);
	float aa = max(fwidth(r), 0.02);

	if (r > 0.96) {
		discard;
	}

	float badge_alpha = 1.0 - smoothstep(0.86 - aa, 0.94 + aa, r);
	float border_ring = 1.0 - smoothstep(0.0, aa * 1.5, abs(r - 0.86) - 0.035);

	float icon_d;
	if (in_icon_kind == 0u) {
		icon_d = sd_camera_icon(in_uv);
	} else if (in_icon_kind == 1u) {
		icon_d = sd_directional_light_icon(in_uv);
	} else if (in_icon_kind == 2u) {
		icon_d = sd_point_light_icon(in_uv);
	} else {
		icon_d = sd_spot_light_icon(in_uv);
	}

	float icon_mask = 1.0 - smoothstep(-aa, aa, icon_d);

	vec3 bg_col = vec3(0.08, 0.09, 0.12);
	vec3 tint = in_color.rgb;
	vec3 col = mix(bg_col, tint, border_ring * 0.85);
	col = mix(col, mix(tint, vec3(1.0), 0.35), icon_mask);

	float alpha = max(badge_alpha * 0.78, max(border_ring * 0.95, icon_mask)) * in_color.a;
	if (alpha < 0.01) {
		discard;
	}

	out_color = vec4(col, alpha);
}
