/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#version 450

#extension GL_EXT_buffer_reference : require


layout(location = 0) out vec4 out_color;

layout(location = 0) in vec3 pos_world;
layout(location = 1) in vec3 normal_world;

#if defined(WITH_COLOR)
	layout(location = 2) in vec3 color;
#elif defined(WITH_TEXTURE)
	layout(location = 2) in vec2 uv;
#endif


#if defined(WITH_TEXTURE)
	layout(binding = 2) uniform sampler2D texSampler;
#endif



void main()
{
	#if defined(WITH_COLOR)
		out_color = vec4(color, 1.0);
	#elif defined(WITH_TEXTURE)
		out_color = texture(texSampler, uv);
	#endif
}
