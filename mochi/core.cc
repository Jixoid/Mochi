/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/core.hh"
#include "mochi/asset/mesh.hh"
#include "mochi/ecs/camera.hh"
#include "mochi/ecs/mesh.hh"
#include "mochi/ecs/camera.hh"
#include "mochi/ecs/point_light.hh"
#include "mochi/ecs/transform.hh"
#include "mochi/rhi/pipeline.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/module/device.hh"
#include "mochi/module/resource.hh"
#include "mochi/module/display.hh"
#include "mochi/module/memory.hh"
#include "mochi/module/renderer.hh"
#include <chrono>
#include <vulkan/vulkan_raii.hpp>



namespace mochi
{

  core::core(
    std::function<vk::raii::PhysicalDevice (vk::raii::PhysicalDevices)> GpuPicker,
    std::function<void (f32 dt)> Idle
  )
    : m_idle(Idle)
  {
    auto bridge     = make_uptr(new module::bridge("Mochi Test", {1,0,0,0}));
    auto device     = make_uptr(new module::device(GpuPicker(bridge->physicalDevices())));
    auto resource = make_uptr(new module::resource(*device));
    auto memory     = make_uptr(new module::memory(*bridge, *device));
    auto display   = make_uptr(new module::display(*bridge, *device, *memory, "Mochi Test", 800, 600));
    auto renderer = make_uptr(new module::renderer(*device));


    m_modules = decltype(m_modules)(
      std::move(bridge),
      std::move(device),
      std::move(resource),
      std::move(memory),
      std::move(display),
      std::move(renderer)
    );
  }

  core::~core()
  {
    sub<module::device>().vdevice().waitIdle();
    
    m_registry.clear();
    
    std::get<uptr<module::renderer>>(m_modules).reset();
    std::get<uptr<module::display>>(m_modules).reset();
    std::get<uptr<module::memory>>(m_modules).reset();
    std::get<uptr<module::resource>>(m_modules).reset();
    std::get<uptr<module::device>>(m_modules).reset();
    std::get<uptr<module::bridge>>(m_modules).reset();
  }




  fun core::run() -> void
  {
    auto last_time = std::chrono::high_resolution_clock::now();
    
    while (sub<module::display>().proc_events()) {
      auto current_time = std::chrono::high_resolution_clock::now();
      float dt = std::chrono::duration<float, std::chrono::seconds::period>(current_time-last_time).count();
      last_time = current_time;
        
      m_idle(dt);

      draw();
    }
  }


  fun core::paint(vk::raii::CommandBuffer &cmd) -> void
  {
    auto &mem = sub<module::memory>();

    // Update camera system
    auto cameras = m_registry.view<ecs::Transform, ecs::Camera>();
    size_t cam_count = std::distance(cameras.begin(), cameras.end());
    auto cam_ubo = mem.camera_ubo(std::max<size_t>(1, cam_count));
    
    if (cam_ubo && cam_ubo->mapped()) {
      auto* cam_data = (ecs::camera_t*)cam_ubo->mapped();
      u32 idx = 0;
      for (auto entity : cameras) {
        auto &cam = cameras.get<ecs::Camera>(entity);
        cam_data[idx].view = cam.view;
        cam_data[idx].proj = cam.proj;
        idx++;
      }
    }

    // Update light system
    auto lights = m_registry.view<ecs::Transform, ecs::PointLight>();
    size_t lig_count = std::distance(lights.begin(), lights.end());
    auto lig_ubo = mem.light_ubo(std::max<size_t>(1, lig_count + 1));

    if (lig_ubo && lig_ubo->mapped()) {
      auto* lig_data = (ecs::point_light_t*)lig_ubo->mapped();
      *((u32*)lig_data) = lig_count; // First 4 bytes hold the count
      
      u32 idx = 1;
      for (auto entity : lights) {
        auto &transform = lights.get<ecs::Transform>(entity);
        auto &light = lights.get<ecs::PointLight>(entity);
        
        lig_data[idx].position = {transform.model.SwVec[0][3], transform.model.SwVec[1][3], transform.model.SwVec[2][3], 0.0f};
        lig_data[idx].color = {light.color, light.intensity};
        idx++;
      }
    }


    auto extent = sub<module::display>().extent(); 
    cmd.setViewport(0, {vk::Viewport(0.0f, 0.0f, (float)extent.width, (float)extent.height, 0.0f, 1.0f)});
    cmd.setScissor(0, {vk::Rect2D({0, 0}, extent)});

    // Render loop
    rhi::pipeline *last_pipeline{};
    rhi::buffer   *last_buffer{};

    auto view = m_registry.view<ecs::Transform, ecs::Mesh>();
    for (auto entity : view)
    {
      auto &transform = view.get<ecs::Transform>(entity);
      auto &renderable = view.get<ecs::Mesh>(entity);

      if (!renderable.material || !renderable.mesh)
        continue;

      auto pipe = renderable.material->get_pipeline();
      auto mesh = renderable.mesh;


      if (pipe.get() != last_pipeline) {
        cmd.bindPipeline(vk::PipelineBindPoint::eGraphics, *pipe->get());
        last_pipeline = pipe.get();
      }


      if (mesh->data().get() != last_buffer) {
        cmd.bindVertexBuffers(0, {mesh->data()->get()}, {0}); 
        last_buffer = mesh->data().get();
      }

      cmd.bindDescriptorSets(
        vk::PipelineBindPoint::eGraphics,
        *pipe->layout(),
        0,
        {*renderable.material->get_desc_set(), },
        {}
      );

      auto model_mat = transform.model;
      cmd.pushConstants<mochi::mat4<f32>>(
        *last_pipeline->layout(),
        vk::ShaderStageFlagBits::eVertex | vk::ShaderStageFlagBits::eFragment,
        0,
        model_mat
      );


      cmd.draw(mesh->data()->size() / asset::vertex_i.stride(), 1, 0, 0);
    }
  }

  fun core::draw() -> void
  {
    auto &ren = sub<module::renderer>();
    auto &disp = sub<module::display>();

    auto &cmd = ren.begin_frame();

    u32 current_frame = ren.current_frame();
    vk::Semaphore wait_sem = ren.get_image_available_sem(current_frame);

    u32 image_index = disp.acquire_next_image(wait_sem);
    auto target = disp.get_render_target(image_index);
    vk::Semaphore signal_sem = disp.get_render_finished_sem(image_index);

    ren.begin_pass(cmd, target, {0.1f, 0.1f, 0.1f, 1.0f});

    /* paint */ paint(cmd);

    ren.end_pass(cmd, target);

    vk::Semaphore w[] = {wait_sem};
    vk::Semaphore s[] = {signal_sem};
    ren.end_frame(cmd, w, s);

    disp.present(signal_sem, image_index);
  }
  
}
