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


struct vertex_t {
  vec3 pos;
  vec3 normal;
  vec3 color;
  vec2 uv;
};

layout(buffer_reference, std430) readonly buffer VertexBuffer {
  vertex_t v[];
};


#if defined(WITH_MULTI_INST)
  struct inst_t {
    vec4 pos;
  };

  layout(buffer_reference, std430) readonly buffer InstBuffer {
    inst_t v[];
  };
#endif


layout(location = 0) out vec3 frag_pos_world;
layout(location = 1) out vec3 frag_normal_world;

#if defined(WITH_COLOR)
  layout(location = 2) out vec3 frag_color;
#elif defined(WITH_TEXTURE)
  layout(location = 2) out vec2 frag_uv;
#endif


layout(buffer_reference, std430, row_major) readonly buffer CameraBuffer {
  mat4 view;
  mat4 proj;
};

layout(push_constant, row_major) uniform PushConstant {
  mat4x3 model;

  VertexBuffer vertexs;
  CameraBuffer camera;
  uint64_t light_addr;
  
  #if defined(WITH_MULTI_INST)
    InstBuffer insts;
  #endif

  uint texture_id;
} push;





void main() {
  vertex_t vex = push.vertexs.v[gl_VertexIndex];

  #if defined(WITH_MULTI_INST)
    inst_t ins = push.insts.v[gl_InstanceIndex];
  #endif
  

  #if defined(WITH_COLOR)
    frag_color = vex.color;
  #elif defined(WITH_TEXTURE)
    frag_uv = vex.uv;
  #endif

  mat4 trueModel = transpose(mat4(push.model));


  vec4 worldPos;
  #if defined(WITH_MULTI_INST)
    worldPos = trueModel * vec4(vex.pos + ins.pos.xyz, 1.0);
  #elif defined(WITH_SINGLE_INST)
    worldPos = trueModel * vec4(vex.pos, 1.0);
  #endif

  frag_pos_world = worldPos.xyz;

  mat3 normalMatrix = transpose(inverse(mat3(trueModel)));
  frag_normal_world = normalize(normalMatrix * vex.normal);

  gl_Position = push.camera.proj * push.camera.view * worldPos;
}
