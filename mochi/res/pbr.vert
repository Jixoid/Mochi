/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#version 450

#extension GL_EXT_buffer_reference : require


layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 color;
layout(location = 3) in vec2 uv;

layout(location = 0) out vec3 frag_pos_world;
layout(location = 1) out vec3 frag_normal_world;

#if defined(WITH_COLOR)
  layout(location = 2) out vec3 frag_color;
#elif defined(WITH_TEXTURE)
  layout(location = 2) out vec2 frag_uv;
#endif


layout(push_constant) uniform PushConstant
{
  mat4 model;
} push;

layout(set = 0, binding = 0, row_major) uniform CameraBuffer
{
  mat4 view;
  mat4 proj;
} camera;



void main()
{
  #if defined(WITH_COLOR)
    frag_color = color;
  #elif defined(WITH_TEXTURE)
    frag_uv = uv;
  #endif

  mat4 trueModel = transpose(push.model);

  vec4 worldPos = trueModel * vec4(pos, 1.0);
  frag_pos_world = worldPos.xyz;

  mat3 normalMatrix = transpose(inverse(mat3(trueModel)));
  frag_normal_world = normalize(normalMatrix * normal);

  gl_Position = camera.proj * camera.view * worldPos;
}
