/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include <cstring>
#include <stdexcept>
#include <iostream>
#include <utility>
#include "Basis.h"
#include "qvk/entity/pipeline.hh"
#include "qvk/entity/object.hh"
#include "qvk/shader.hh"
#include "qvk/core.hh"
#include "qvk/geometry.hh"
#include "vulkan/vulkan.hpp"



using namespace qvk;


struct vertex
{
  vec3<f32> pos;
  vec3<f32> color;
  vec3<f32> inNormal;
  vec2<f32> uv;
  f32 __pad;
};

std::vector<vertex> cube_vertices {
  // Yüz 1 (Üst - Y eksenine bakar: 0, 1, 0)
  {{ 1.0f,  1.0f, -1.0f}, {1.0f, 0.0f, 0.0f}, { 0.0f,  1.0f,  0.0f}, {0.625f, 0.5f}},
  {{-1.0f,  1.0f, -1.0f}, {1.0f, 0.0f, 0.0f}, { 0.0f,  1.0f,  0.0f}, {0.875f, 0.5f}},
  {{-1.0f,  1.0f,  1.0f}, {1.0f, 0.0f, 0.0f}, { 0.0f,  1.0f,  0.0f}, {0.875f, 0.75f}},
  {{ 1.0f,  1.0f, -1.0f}, {1.0f, 0.0f, 0.0f}, { 0.0f,  1.0f,  0.0f}, {0.625f, 0.5f}},
  {{-1.0f,  1.0f,  1.0f}, {1.0f, 0.0f, 0.0f}, { 0.0f,  1.0f,  0.0f}, {0.875f, 0.75f}},
  {{ 1.0f,  1.0f,  1.0f}, {1.0f, 0.0f, 0.0f}, { 0.0f,  1.0f,  0.0f}, {0.625f, 0.75f}},

  // Yüz 2 (Sağ - X eksenine bakar: 1, 0, 0)  *(Not: Pozisyonlara göre aslında Z eksenine bakıyor)*
  {{ 1.0f, -1.0f,  1.0f}, {0.0f, 1.0f, 0.0f},  { 0.0f,  0.0f,  1.0f}, {0.375f, 0.75f}},
  {{ 1.0f,  1.0f,  1.0f}, {0.0f, 1.0f, 0.0f},  { 0.0f,  0.0f,  1.0f}, {0.625f, 0.75f}},
  {{-1.0f,  1.0f,  1.0f}, {0.0f, 1.0f, 0.0f},  { 0.0f,  0.0f,  1.0f}, {0.625f, 1.0f}},
  {{ 1.0f, -1.0f,  1.0f}, {0.0f, 1.0f, 0.0f},  { 0.0f,  0.0f,  1.0f}, {0.375f, 0.75f}},
  {{-1.0f,  1.0f,  1.0f}, {0.0f, 1.0f, 0.0f}, { 0.0f,  0.0f,  1.0f}, {0.625f, 1.0f}},
  {{-1.0f, -1.0f,  1.0f}, {0.0f, 1.0f, 0.0f}, { 0.0f,  0.0f,  1.0f}, {0.375f, 1.0f}},

  // Yüz 3 (Arka - Negatif X eksenine bakar: -1, 0, 0)
  {{-1.0f, -1.0f,  1.0f}, {0.0f, 0.0f, 1.0f}, {-1.0f,  0.0f,  0.0f}, {0.375f, 0.0f}},
  {{-1.0f,  1.0f,  1.0f}, {0.0f, 0.0f, 1.0f}, {-1.0f,  0.0f,  0.0f}, {0.625f, 0.0f}},
  {{-1.0f,  1.0f, -1.0f}, {0.0f, 0.0f, 1.0f}, {-1.0f,  0.0f,  0.0f}, {0.625f, 0.25f}},
  {{-1.0f, -1.0f,  1.0f}, {0.0f, 0.0f, 1.0f}, {-1.0f,  0.0f,  0.0f}, {0.375f, 0.0f}},
  {{-1.0f,  1.0f, -1.0f}, {0.0f, 0.0f, 1.0f}, {-1.0f,  0.0f,  0.0f}, {0.625f, 0.25f}},
  {{-1.0f, -1.0f, -1.0f}, {0.0f, 0.0f, 1.0f}, {-1.0f,  0.0f,  0.0f}, {0.375f, 0.25f}},

  // Yüz 4 (Sol - Negatif Y eksenine bakar: 0, -1, 0)
  {{-1.0f, -1.0f, -1.0f}, {1.0f, 1.0f, 0.0f}, { 0.0f, -1.0f,  0.0f}, {0.125f, 0.5f}},
  {{ 1.0f, -1.0f, -1.0f}, {1.0f, 1.0f, 0.0f}, { 0.0f, -1.0f,  0.0f}, {0.375f, 0.5f}},
  {{ 1.0f, -1.0f,  1.0f}, {1.0f, 1.0f, 0.0f}, { 0.0f, -1.0f,  0.0f}, {0.375f, 0.75f}},
  {{-1.0f, -1.0f, -1.0f}, {1.0f, 1.0f, 0.0f}, { 0.0f, -1.0f,  0.0f}, {0.125f, 0.5f}},
  {{ 1.0f, -1.0f,  1.0f}, {1.0f, 1.0f, 0.0f}, { 0.0f, -1.0f,  0.0f}, {0.375f, 0.75f}},
  {{-1.0f, -1.0f,  1.0f}, {1.0f, 1.0f, 0.0f}, { 0.0f, -1.0f,  0.0f}, {0.125f, 0.75f}},

  // Yüz 5 (Alt - Pozitif X eksenine bakar: 1, 0, 0) *(Pozisyonlara göre)*
  {{ 1.0f, -1.0f, -1.0f}, {1.0f, 0.0f, 1.0f}, { 1.0f,  0.0f,  0.0f}, {0.375f, 0.5f}},
  {{ 1.0f,  1.0f, -1.0f}, {1.0f, 0.0f, 1.0f}, { 1.0f,  0.0f,  0.0f}, {0.625f, 0.5f}},
  {{ 1.0f,  1.0f,  1.0f}, {1.0f, 0.0f, 1.0f}, { 1.0f,  0.0f,  0.0f}, {0.625f, 0.75f}},
  {{ 1.0f, -1.0f, -1.0f}, {1.0f, 0.0f, 1.0f}, { 1.0f,  0.0f,  0.0f}, {0.375f, 0.5f}},
  {{ 1.0f,  1.0f,  1.0f}, {1.0f, 0.0f, 1.0f}, { 1.0f,  0.0f,  0.0f}, {0.625f, 0.75f}},
  {{ 1.0f, -1.0f,  1.0f}, {1.0f, 0.0f, 1.0f}, { 1.0f,  0.0f,  0.0f}, {0.375f, 0.75f}},

  // Yüz 6 (Ön - Negatif Z eksenine bakar: 0, 0, -1) *(Pozisyonlara göre)*
  {{-1.0f, -1.0f, -1.0f}, {0.0f, 1.0f, 1.0f}, { 0.0f,  0.0f, -1.0f}, {0.125f, 0.25f}},
  {{-1.0f,  1.0f, -1.0f}, {0.0f, 1.0f, 1.0f}, { 0.0f,  0.0f, -1.0f}, {0.375f, 0.25f}},
  {{ 1.0f,  1.0f, -1.0f}, {0.0f, 1.0f, 1.0f}, { 0.0f,  0.0f, -1.0f}, {0.375f, 0.5f}},
  {{-1.0f, -1.0f, -1.0f}, {0.0f, 1.0f, 1.0f}, { 0.0f,  0.0f, -1.0f}, {0.125f, 0.25f}},
  {{ 1.0f,  1.0f, -1.0f}, {0.0f, 1.0f, 1.0f}, { 0.0f,  0.0f, -1.0f}, {0.375f, 0.5f}},
  {{ 1.0f, -1.0f, -1.0f}, {0.0f, 1.0f, 1.0f}, { 0.0f,  0.0f, -1.0f}, {0.125f, 0.5f}},
};



  
int Main()
{
  core core(
    [](vk::raii::PhysicalDevices devices) -> vk::raii::PhysicalDevice {

      if (devices.empty())
        throw std::runtime_error("Vulkan destekli bir grafik birimi bulunamadı.");

      return devices[0];
    },
    []()
    {
      //auto dir = (vec3<f32>{-10,-10,-10} - vec3<f32>{0,0,0}).normalize();
      //
      //cam->rot() = quaternion<f32>::fromAxisAngle(0.05, dir) * cam->rot();
      //cam->recalc();
    }
  );


  auto vertex_i = info<buffer>::make(core,
    sizeof(vertex),
    gt::make_list<
      vec3<f32>, // pos
      vec3<f32>, // color
      vec3<f32>, // inNormal
      vec2<f32>  // uv
    >()
  );

  auto object_i = info<buffer>::make(core,
    sizeof(mat4<f32>),
    gt::make_list<
      mat4<f32> // model
    >()
  );
  
  auto ucam_i = info<buffer>::make(core,
    sizeof(mat4<f32>)*2,
    gt::make_list<
      mat4<f32>, // view
      mat4<f32>  // proj
    >()
  );

  auto pbr_i = info<pipeline>::make(core,
    {
      //{gt::make<mat4<f32>>(), vk::ShaderStageFlagBits::eVertex},
    },
    {
      {vertex_i, vk::VertexInputRate::eVertex},
      {object_i, vk::VertexInputRate::eInstance},
    },
    {
      {ucam_i, vk::ShaderStageFlagBits::eVertex | vk::ShaderStageFlagBits::eFragment},
    }
  );

  core.sub<meta>().compile();



  std::vector<shaderSlot> shaders;
  shaders.push_back(shaderSlot{vk::ShaderStageFlagBits::eVertex,   shader(core, "pkgs/com.test.qvk/res/cube.vert.spv", "main")});
  shaders.push_back(shaderSlot{vk::ShaderStageFlagBits::eFragment, shader(core, "pkgs/com.test.qvk/res/cube.frag.spv", "main")});
  
  auto pbr = pipeline::make(core, pbr_i,
    std::move(shaders)
  );

  auto vertexs = core.sub<memory>().load_VertexBuffer(
    vertex_i,
    cube_vertices.size(),
    [](void *dest) -> void
    {
      memcpy(dest, cube_vertices.data(), sizeof(vertex) * cube_vertices.size());
    }
  );

  auto objects = core.sub<memory>().load_VertexBuffer(
    object_i,
    2,
    [](void *dest) -> void
    {
      auto models = (mat4<f32>*)dest;


      models[0] = mat4<f32>::model(
        {10, 10, 10},
        quaternion<f32>{},
        {1,1,1}
      );
      
      models[1] = mat4<f32>::model(
        {0, 0, 0},
        quaternion<f32>{},
        {1,1,1}
      );

    }
  );

  auto cam = core.sub<memory>().load_UniformBuffer(
    ucam_i,
    1,
    [](void *dest) -> void
    {
      auto models = (mat4<f32>*)dest;


      vec3<f32> cameraPos   = {20, 20, 20};
      vec3<f32> cameraTarget = {0, 0, 0};
      vec3<f32> cameraUp    = {0, 1, 0};

      models[0] = mat4<f32>::lookAt(cameraPos, cameraTarget, cameraUp);

      models[1] = mat4<f32>::perspective(
        45,
        800.0/600.0,
        0.1,
        1000
      );
    }
  );
  
 
  auto acube = object::make(core, pbr,
    {
      {vertexs, vk::VertexInputRate::eVertex},
      {objects, vk::VertexInputRate::eInstance},
    },
    {
      {cam, vk::ShaderStageFlagBits::eVertex | vk::ShaderStageFlagBits::eFragment}
    },
    cube_vertices.size(), 2
  );



  core.run();

  return 0;
}




int main()
{
  try { return Main(); }
  
  catch (const vk::SystemError &e) {
    std::cerr << "Vulkan Hatasi: " << e.what() << std::endl;
    return -1;
  }
  catch (const std::exception &e) {
    std::cerr << "Standart Hata: " << e.what() << std::endl;
    return -1;
  }

  return 0;
}
