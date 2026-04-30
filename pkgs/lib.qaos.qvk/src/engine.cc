/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "qvk/engine.hh"
#include "Basis.hh"
#include "qvk/buffer.hh"
#include "qvk/device.hh"
#include "qvk/memory.hh"
#include "qvk/geometry.hh"
#include "qvk/object.hh"
#include "qvk/camera.hh"
#include "qvk/pipeline.hh"
#include "qvk/swapchain.hh"
#include <vulkan/vulkan_raii.hpp>



namespace qvk
{

  engine::engine(
    std::function<vk::raii::PhysicalDevice (vk::raii::PhysicalDevices)> GpuPicker,
    std::function<void ()> Idle
  )
    : m_bridge("QVK Test", {1,0,0,0})
    , m_window(m_bridge, "QVK Test", 800, 600)
    , m_device(GpuPicker(m_bridge.physicalDevices()))
    , m_swapchain(m_device, m_window)
    , m_renderer(m_device, m_swapchain)
    , m_memory(m_device, m_renderer)
    , m_meta(m_device)

    , m_idle(Idle)
  {}

  engine::~engine()
  {
    // Close GPU
    m_device.vdevice().waitIdle();
  }




  fun engine::run() -> void
  {
    while (m_window.proc_events()) {
        
      // İleride buraya eklenecekler:
      // update_input();
      // update_physics();
      // update_camera();

      m_idle();

      draw();
    }
  }


  fun engine::draw() -> void
  {
    auto &cmd = m_renderer.begin_frame();
    m_renderer.begin_swapchain_rendering(cmd, {0.1f, 0.1f, 0.1f, 1.0f});

    // 2. Dinamik Viewport ve Scissor (Karede 1 kez yapılması yeterlidir, döngüye girmez!)
    auto extent = m_swapchain.extent(); 
    cmd.setViewport(0, {vk::Viewport(0.0f, 0.0f, (float)extent.width, (float)extent.height, 0.0f, 1.0f)});
    cmd.setScissor(0, {vk::Rect2D({0, 0}, extent)});

    // --- BİRDEN FAZLA NESNEYİ ÇİZME DÖNGÜSÜ ---
    
    qvk::pipeline* last_pipeline = nullptr;
    qvk::buffer* last_buffer   = nullptr;

    for (const auto &obj: sub<memory>().list<object>())
    {  
      // A. Eğer bu nesnenin pipeline'ı bir öncekinden farklıysa yenisini bağla
      if (obj->pipeline() != last_pipeline) {
        cmd.bindPipeline(vk::PipelineBindPoint::eGraphics, *obj->pipeline()->get());
        last_pipeline = obj->pipeline();
      }

      // B. Eğer vertex buffer değiştiyse yenisini bağla
      if (obj->buffer() != last_buffer) {
        vk::DeviceSize offset = 0;
        cmd.bindVertexBuffers(0, {*obj->buffer()->get()}, {offset});
        last_buffer = obj->buffer();
      }


      // Push Constant
      cmd.pushConstants<qvk::mat4<f32>>(
        last_pipeline->layout(),
        vk::ShaderStageFlags::BitsType::eVertex,
        0,
        obj->camera()->proj() * obj->camera()->view() * obj->model()
      );


      // Draw
      cmd.draw(obj->count(), 1, 0, 0);
    }

    // 3. Tuvali Kapat ve Ekrana Gönder
    m_renderer.end_swapchain_rendering(cmd);
    m_renderer.end_frame(cmd);
  }
  
}
