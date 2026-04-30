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
#include "qvk/camera.hh"
#include "qvk/engine.hh"
#include "qvk/geometry.hh"
#include "qvk/meta.hh"
#include "qvk/object.hh"
#include "qvk/pipeline.hh"
#include "qvk/shader.hh"




std::vector<qvk::vertex> cube_vertices {
  // Yüz 1 (Üst/Ön)
  {{ 1.0f,  1.0f, -1.0f}, {1.0f, 0.0f, 0.0f}, {0.625f, 0.5f}},
  {{-1.0f,  1.0f, -1.0f}, {1.0f, 0.0f, 0.0f}, {0.875f, 0.5f}},
  {{-1.0f,  1.0f,  1.0f}, {1.0f, 0.0f, 0.0f}, {0.875f, 0.75f}},
  {{ 1.0f,  1.0f, -1.0f}, {1.0f, 0.0f, 0.0f}, {0.625f, 0.5f}},
  {{-1.0f,  1.0f,  1.0f}, {1.0f, 0.0f, 0.0f}, {0.875f, 0.75f}},
  {{ 1.0f,  1.0f,  1.0f}, {1.0f, 0.0f, 0.0f}, {0.625f, 0.75f}},

  // Yüz 2 (Sağ)
  {{ 1.0f, -1.0f,  1.0f}, {0.0f, 1.0f, 0.0f}, {0.375f, 0.75f}},
  {{ 1.0f,  1.0f,  1.0f}, {0.0f, 1.0f, 0.0f}, {0.625f, 0.75f}},
  {{-1.0f,  1.0f,  1.0f}, {0.0f, 1.0f, 0.0f}, {0.625f, 1.0f}},
  {{ 1.0f, -1.0f,  1.0f}, {0.0f, 1.0f, 0.0f}, {0.375f, 0.75f}},
  {{-1.0f,  1.0f,  1.0f}, {0.0f, 1.0f, 0.0f}, {0.625f, 1.0f}},
  {{-1.0f, -1.0f,  1.0f}, {0.0f, 1.0f, 0.0f}, {0.375f, 1.0f}},

  // Yüz 3 (Arka)
  {{-1.0f, -1.0f,  1.0f}, {0.0f, 0.0f, 1.0f}, {0.375f, 0.0f}},
  {{-1.0f,  1.0f,  1.0f}, {0.0f, 0.0f, 1.0f}, {0.625f, 0.0f}},
  {{-1.0f,  1.0f, -1.0f}, {0.0f, 0.0f, 1.0f}, {0.625f, 0.25f}},
  {{-1.0f, -1.0f,  1.0f}, {0.0f, 0.0f, 1.0f}, {0.375f, 0.0f}},
  {{-1.0f,  1.0f, -1.0f}, {0.0f, 0.0f, 1.0f}, {0.625f, 0.25f}},
  {{-1.0f, -1.0f, -1.0f}, {0.0f, 0.0f, 1.0f}, {0.375f, 0.25f}},

  // Yüz 4 (Sol)
  {{-1.0f, -1.0f, -1.0f}, {1.0f, 1.0f, 0.0f}, {0.125f, 0.5f}},
  {{ 1.0f, -1.0f, -1.0f}, {1.0f, 1.0f, 0.0f}, {0.375f, 0.5f}},
  {{ 1.0f, -1.0f,  1.0f}, {1.0f, 1.0f, 0.0f}, {0.375f, 0.75f}},
  {{-1.0f, -1.0f, -1.0f}, {1.0f, 1.0f, 0.0f}, {0.125f, 0.5f}},
  {{ 1.0f, -1.0f,  1.0f}, {1.0f, 1.0f, 0.0f}, {0.375f, 0.75f}},
  {{-1.0f, -1.0f,  1.0f}, {1.0f, 1.0f, 0.0f}, {0.125f, 0.75f}},

  // Yüz 5 (Alt)
  {{ 1.0f, -1.0f, -1.0f}, {1.0f, 0.0f, 1.0f}, {0.375f, 0.5f}},
  {{ 1.0f,  1.0f, -1.0f}, {1.0f, 0.0f, 1.0f}, {0.625f, 0.5f}},
  {{ 1.0f,  1.0f,  1.0f}, {1.0f, 0.0f, 1.0f}, {0.625f, 0.75f}},
  {{ 1.0f, -1.0f, -1.0f}, {1.0f, 0.0f, 1.0f}, {0.375f, 0.5f}},
  {{ 1.0f,  1.0f,  1.0f}, {1.0f, 0.0f, 1.0f}, {0.625f, 0.75f}},
  {{ 1.0f, -1.0f,  1.0f}, {1.0f, 0.0f, 1.0f}, {0.375f, 0.75f}},

  // Yüz 6 (Ön)
  {{-1.0f, -1.0f, -1.0f}, {0.0f, 1.0f, 1.0f}, {0.125f, 0.25f}},
  {{-1.0f,  1.0f, -1.0f}, {0.0f, 1.0f, 1.0f}, {0.375f, 0.25f}},
  {{ 1.0f,  1.0f, -1.0f}, {0.0f, 1.0f, 1.0f}, {0.375f, 0.5f}},
  {{-1.0f, -1.0f, -1.0f}, {0.0f, 1.0f, 1.0f}, {0.125f, 0.25f}},
  {{ 1.0f,  1.0f, -1.0f}, {0.0f, 1.0f, 1.0f}, {0.375f, 0.5f}},
  {{ 1.0f, -1.0f, -1.0f}, {0.0f, 1.0f, 1.0f}, {0.125f, 0.5f}}
};


using namespace qvk;


  
int Main()
{
  camera *cam{};


  engine eng(
    [](vk::raii::PhysicalDevices devices) -> vk::raii::PhysicalDevice {

      if (devices.empty())
        throw std::runtime_error("Vulkan destekli bir grafik birimi bulunamadı.");

      return devices[0];
    },
    [&cam]()
    {
      auto dir = (vec3<f32>{-10,-10,-10} - vec3<f32>{0,0,0}).normalize();

      cam->rot() = quaternion<f32>::fromAxisAngle(0.1, dir) * cam->rot();
      cam->recalc();
    }
  );

  
  auto scene_1 = eng.sub<meta>().make<scene>();

  auto cam_1 = eng.sub<meta>().make<camera>();
  scene_1->add(cam_1);
  
  
  
  cam = camera::make(eng,
    {-10,-10,-10},
    {},
    {0,1,0},
    45, 0.1, 1000,
    cam_1
  );


  auto pbr = pipeline::make(eng,
    shader(eng, "pkgs/com.test.qvk/res/cube.vert.spv"),
    shader(eng, "pkgs/com.test.qvk/res/cube.frag.spv")
  );


  auto data = eng.sub<memory>().load_VertexBuffer(
    sizeof(vertex) * cube_vertices.size(),
    [](void *dest) -> void
    {
      memcpy(dest, cube_vertices.data(), sizeof(vertex) * cube_vertices.size());
    }
  );
  
  
  auto acube = object::make(eng,
    mat4<f32>::model({20, 20, 20}, quaternion<f32>{}, {1,1,1}),
    cam,
    pbr,
    data,
    64
  );



  eng.run();

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
