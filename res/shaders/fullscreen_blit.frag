#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D u_textures[];

layout(push_constant) uniform FullscreenBlitPushConstants {
	uint texture_id;
	uint _pad0;
	uint _pad1;
	uint _pad2;
} pc;

void main() {
	out_color = texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv);
}
