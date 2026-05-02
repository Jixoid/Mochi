/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "Basis.hh"
#include "qvk/core.hh"
#include "qvk/module/device.hh"
#include "qvk/module/memory.hh"
#include "qvk/module/swapchain.hh"
#include "qvk/entity/pipeline.hh"
#include "qvk/entity/buffer.hh"
#include "qvk/geometry.hh"
#include "qvk/entity/object.hh"
#include <chrono>
#include <vulkan/vulkan_raii.hpp>



namespace qvk
{

  core::core(
    std::function<vk::raii::PhysicalDevice (vk::raii::PhysicalDevices)> GpuPicker,
    std::function<void ()> Idle
  )
    : m_bridge("QVK Test", {1,0,0,0})
    , m_window(m_bridge, "QVK Test", 1920, 1080)
    , m_device(GpuPicker(m_bridge.physicalDevices()))
    , m_swapchain(m_device, m_window)
    , m_renderer(m_device, m_swapchain)
    , m_memory(m_device, m_renderer)
    , m_meta(m_memory)

    , m_idle(Idle)
  {}

  core::~core()
  {
    // Close GPU
    m_device.vdevice().waitIdle();
  }




  fun core::run() -> void
  {
    auto last_time = std::chrono::high_resolution_clock::now();
    
    while (m_window.proc_events()) {
      auto current_time = std::chrono::high_resolution_clock::now();
      float dt = std::chrono::duration<float, std::chrono::seconds::period>(current_time-last_time).count();
      last_time = current_time;
        
      // İleride buraya eklenecekler:
      // update_input();
      // update_physics();
      // update_camera();

      m_idle();

      draw();
    }
  }


  fun core::draw() -> void
  {
    auto &cmd = m_renderer.begin_frame();
    m_renderer.begin_swapchain_rendering(cmd, {0.1f, 0.1f, 0.1f, 1.0f});

    // 2. Dinamik Viewport ve Scissor (Karede 1 kez yapılması yeterlidir, döngüye girmez!)
    auto extent = m_swapchain.extent(); 
    cmd.setViewport(0, {vk::Viewport(0.0f, 0.0f, (float)extent.width, (float)extent.height, 0.0f, 1.0f)});
    cmd.setScissor(0, {vk::Rect2D({0, 0}, extent)});

    // --- BİRDEN FAZLA NESNEYİ ÇİZME DÖNGÜSÜ ---
    qvk::pipeline* last_pipeline{};
    //qvk::buffer* last_buffer{};

    for (const auto &obj: sub<memory>().list<object>())
    {
      // 1. Pipeline Bağla
      if (obj->pipeline() != last_pipeline) {
        cmd.bindPipeline(vk::PipelineBindPoint::eGraphics, *obj->pipeline()->get());
        last_pipeline = obj->pipeline();
      }

      // 2. Vertex ve Instance Buffer'ları Bağla
      u32 i{};
      for (auto &X: obj->vertexs()) {
        // Vulkan array proxy beklediği için süslü parantez içine alıyoruz
        cmd.bindVertexBuffers(i, {*X.buf()->get()}, {0}); 
        i++;
      }

      // --- YENİ: DESCRIPTOR SET (UBO) BAĞLAMA ---
      if (!obj->uniforms().empty()) {
        cmd.bindDescriptorSets(
          vk::PipelineBindPoint::eGraphics,
          *obj->pipeline()->layout(),
          0,                       // firstSet (0'dan başlıyor)
          {*obj->desc_sets()[0]},  // Bizim hazırladığımız paket
          {}                       // dynamicOffsets (boş)
        );
      }

      // 3. Draw Komutu (Dinamik)
      cmd.draw(obj->vertex_count(), obj->instance_count(), 0, 0);
    }

    // 3. Tuvali Kapat ve Ekrana Gönder
    m_renderer.end_swapchain_rendering(cmd);
    m_renderer.end_frame(cmd);
  }
  
}
