/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#version 450

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require


layout(location = 0) out vec4 out_color;

layout(location = 0) in vec3 pos_world;
layout(location = 1) in vec3 normal_world;

#if defined(WITH_COLOR)
	layout(location = 2) in vec3 color;
#elif defined(WITH_TEXTURE)
	layout(location = 2) in vec2 uv;
#endif


layout(buffer_reference, std430, row_major) readonly buffer CameraBuffer {
	mat4 view;
	mat4 proj;
};


struct light_t {
	vec4 pos;
	vec4 color;
};

layout(buffer_reference, std430) readonly buffer LightBuffer {
	uvec4 a;
	uvec4 b;
	light_t s[32];
};

layout(push_constant, row_major) uniform PushConstant {
  mat4x3 model;
	
  uint64_t vertex_addr;
  CameraBuffer camera;
  LightBuffer  light;
  
  #if defined(WITH_MULTI_INST)
    uint64_t inst_addr;
  #endif

  uint texture_id;
} push;

#if defined(WITH_TEXTURE)
	layout(set = 1, binding = 0) uniform sampler2D texSampler;
#endif


void main()
{
	#if defined(WITH_COLOR)
		out_color = vec4(color, 1.0);
	#elif defined(WITH_TEXTURE)
		out_color = texture(texSampler, uv);
	#endif
}
