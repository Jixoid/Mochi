/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/core.hh"
#include "mochi/ecs/camera.hh"
#include "mochi/ecs/mesh.hh"
#include "mochi/ecs/camera.hh"
#include "mochi/ecs/multi_mesh.hh"
#include "mochi/ecs/point_light.hh"
#include "mochi/ecs/transform.hh"
#include "mochi/rhi/cmd.hh"
#include "mochi/rhi/image.hh"
#include "mochi/rhi/listPush.hh"
#include "mochi/rhi/pipeline.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/rhi/device.hh"
#include "mochi/module/resource.hh"
#include "mochi/module/display.hh"
#include "mochi/module/memory.hh"
#include "mochi/module/renderer.hh"
#include "mochi/rhi/slotPush.hh"
#include "vulkan/vulkan.hpp"
#include <chrono>
#include <vulkan/vulkan_raii.hpp>



namespace mochi
{

  core::core(
    std::function<i32 (const vk::raii::PhysicalDevices&, rhi::PhysicalDeviceSuitable)> GpuPicker,
    std::function<void (f32 dt)> Idle
  )
    : m_idle(Idle)
  {
    rhi::vulkan_extension vkext{
      .instance_extensions = module::display::extensions(),
    };
    auto device     = make_uptr(new rhi::device("Mochi Test", {1,0,0,0}, GpuPicker, &vkext));

    auto memory     = make_uptr(new module::memory(*device));
    auto resource = make_uptr(new module::resource(*device, *memory));
    auto display   = make_uptr(new module::display(*device, *memory, "Mochi Test", 800, 600));
    auto renderer = make_uptr(new module::renderer(*device));


    m_modules = decltype(m_modules)(
      std::move(device),
      std::move(resource),
      std::move(memory),
      std::move(display),
      std::move(renderer)
    );
  }

  core::~core()
  {
    sub<rhi::device>().get().waitIdle();
    
    m_registry.clear();

    std::get<uptr<rhi::device>>(m_modules).reset();
    
    std::get<uptr<module::renderer>>(m_modules).reset();
    std::get<uptr<module::display>>(m_modules).reset();
    std::get<uptr<module::memory>>(m_modules).reset();
    std::get<uptr<module::resource>>(m_modules).reset();
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


  fun core::paint(rhi::cmd &cmd, rhi::render_target &target) -> void
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
        lig_data[idx].color = vec4{light.color, light.intensity};
        idx++;
      }
    }


    auto extent = sub<module::display>().extent(); 
    cmd.get().setViewport(0, {vk::Viewport(0.0f, 0.0f, (float)extent.width, (float)extent.height, 0.0f, 1.0f)});
    cmd.get().setScissor(0, {vk::Rect2D({0, 0}, extent)});

    

    // Render loop
    rhi::pipeline *last_pipeline{};
    rhi::buffer   *last_buffer{};
    rhi::image2   *last_image{};


    auto view = m_registry.view<ecs::Transform, ecs::Mesh>();
    for (auto entity: view)
    {
      auto &transform = view.get<ecs::Transform>(entity);
      auto &renderable = view.get<ecs::Mesh>(entity);

      if (!renderable.mesh)
        continue;

      auto mesh = renderable.mesh;

      
      u32 i{};
      for (auto &subsur: mesh->offs())
      {
        auto material = renderable.mesh->material()[renderable.mesh->map()[i]];
        auto desc = material->desc(target);

        if (desc->pipeline.get() != last_pipeline) {
          cmd.setPipeline(desc->pipeline.get());
          last_pipeline = desc->pipeline.get();
        }

        cmd.get().bindDescriptorSets(
          vk::PipelineBindPoint::eGraphics,
          *desc->pipeline->layout(),
          0,
          {*desc->desc_sets[0]},
          {}
        );


        if (material->is_texture()) {
          cmd.get().bindDescriptorSets(
            vk::PipelineBindPoint::eGraphics,
            *desc->pipeline->layout(),
            1,
            {*desc->desc_sets[1]},
            {}
          );
        }

        cmd.writePushConstant(rhi::listPush(std::tuple{mat4x4<f32>{transform.model}, mesh->data()->address()}));
        

        cmd.draw({subsur.off(), subsur.size()}, {0, 1});
        i++;
      }

    }


    auto views = m_registry.view<ecs::Transform, ecs::MultiMesh>();
    for (auto entity: views)
    {
      auto &transform = view.get<ecs::Transform>(entity);
      auto &renderable = views.get<ecs::MultiMesh>(entity);

      if (!renderable.mesh)
        continue;

      auto mesh = renderable.mesh;

      
      u32 i{};
      for (auto &subsur: mesh->offs())
      {
        auto material = renderable.mesh->material()[renderable.mesh->map()[i]];
        auto desc = material->desc(target);

        if (desc->pipeline.get() != last_pipeline) {
          cmd.setPipeline(desc->pipeline.get());
          last_pipeline = desc->pipeline.get();
        }

        cmd.get().bindDescriptorSets(
          vk::PipelineBindPoint::eGraphics,
          *desc->pipeline->layout(),
          0,
          {*desc->desc_sets[0]},
          {}
        );


        if (material->is_texture()) {
          cmd.get().bindDescriptorSets(
            vk::PipelineBindPoint::eGraphics,
            *desc->pipeline->layout(),
            1,
            {*desc->desc_sets[1]},
            {}
          );
        }

        cmd.writePushConstant(rhi::listPush(std::tuple{mat4x4<f32>{transform.model}, mesh->data()->address(), renderable.instances->address()}));
        

        cmd.draw({subsur.off(), subsur.size()}, {0, renderable.active_count});
        i++;
      }

    }

  }


  fun core::draw() -> void
  {
    auto &dev = sub<rhi::device>();
    auto &ren = sub<module::renderer>();
    auto &disp = sub<module::display>();


    dev.flushTransferBuf();


    auto &cmd = ren.begin_frame();

    u32 current_frame = ren.current_frame();
    vk::Semaphore wait_sem = ren.get_image_available_sem(current_frame);

    u32 image_index = disp.acquire_next_image(wait_sem);
    auto target = disp.get_render_target(image_index);
    vk::Semaphore signal_sem = disp.get_render_finished_sem(image_index);

    ren.begin_pass(cmd, target, {0.1f, 0.1f, 0.1f, 1.0f});

    rhi::cmd mochi_cmd(&cmd);

    /* paint */ paint(mochi_cmd, target);

    ren.end_pass(cmd, target);

    vk::Semaphore w[] = {wait_sem};
    vk::Semaphore s[] = {signal_sem};
    ren.end_frame(cmd, w, s);

    disp.present(signal_sem, image_index);
  }
  
}
