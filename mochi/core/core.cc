/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/core/core.hh"
#include "mochi/ecs/camera.hh"
#include "mochi/ecs/mesh.hh"
#include "mochi/ecs/multi_mesh.hh"
#include "mochi/ecs/point_light.hh"
#include "mochi/ecs/transform.hh"
#include "mochi/manager/render_manager.hh"
#include "mochi/manager/scene_manager.hh"
#include "mochi/manager/window_manager.hh"
#include "mochi/rhi/command.hh"
#include "mochi/rhi/image.hh"
#include "mochi/rhi/pipeline.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/rhi/render_target.hh"
#include <chrono>
#include <format>
#include <iostream>



namespace mochi
{

  extern "C" {
    extern const char* MochiRHI_DriverInfo;
    extern const u32   MochiRHI_DriverVers[4];
  }


  Core::Core() {
    std::cerr << std::format("Mochi Engine starting {{rhi: {}}}", MochiRHI_DriverInfo) << std::endl;

    auto device = rhi::DeviceManager::make("Mochi Test", {1,0,0,0}, [](){return 0;});
    auto alloc   = rhi::AllocManager::make(*device);
    
    // Initialize descriptor heap for bindless
    device->initDescriptorHeap(*alloc);
    
    auto transfer = rhi::TransferManager::make(*device);

    auto memory   = make_uptr(new manager::SceneManager(*device, *alloc));
    auto shader   = rhi::ShaderManager::make(*device);
    auto pipeline   = rhi::PipelineManager::make(*device);
    auto material = rhi::MaterialManager::make(*device, *shader, *pipeline);
    auto display  = make_uptr(new manager::WindowManager(*device, *memory, "Mochi Test", 800, 600));
    auto renderer = make_uptr(new manager::RenderManager(*device));

    m_modules = decltype(m_modules)(
      std::move(device),
      std::move(alloc),
      std::move(transfer),
      std::move(shader),
      std::move(pipeline),
      std::move(material),
      std::move(memory),
      std::move(display),
      std::move(renderer)
    );
  }

  Core::~Core() {
    sub<rhi::DeviceManager>().waitIdle();
    
    m_registry.clear();

    std::get<uptr<manager::RenderManager>>(m_modules).reset();
    std::get<uptr<manager::WindowManager>>(m_modules).reset();
    std::get<uptr<manager::SceneManager>>(m_modules).reset();
    std::get<uptr<rhi::MaterialManager>>(m_modules).reset();
    std::get<uptr<rhi::PipelineManager>>(m_modules).reset();
    std::get<uptr<rhi::ShaderManager>>(m_modules).reset();
    std::get<uptr<rhi::TransferManager>>(m_modules).reset();
    std::get<uptr<rhi::AllocManager>>(m_modules).reset();
    std::get<uptr<rhi::DeviceManager>>(m_modules).reset();
  }




  fun Core::run() -> void {
    auto last_time = std::chrono::high_resolution_clock::now();
    
    while (sub<manager::WindowManager>().proc_events()) {
      auto current_time = std::chrono::high_resolution_clock::now();
      float dt = std::chrono::duration<float, std::chrono::seconds::period>(current_time-last_time).count();
      last_time = current_time;
        
      m_idle(dt);

      draw();
    }
  }


  fun Core::paint(rhi::Command &cmd, rhi::RenderTarget &target) -> void {
    auto &mem = sub<manager::SceneManager>();

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
      for (auto entity: lights) {
        auto &transform = lights.get<ecs::Transform>(entity);
        auto &light = lights.get<ecs::PointLight>(entity);
        
        lig_data[idx].position = {transform.model.SwVec[0][3], transform.model.SwVec[1][3], transform.model.SwVec[2][3], 0.0f};
        lig_data[idx].color = vec4{light.color, light.intensity};
        idx++;
      }
    }


    auto extent = sub<manager::WindowManager>().extent(); 
    cmd.setViewport(0, {rhi::Viewport(0, (f32)extent.x(), 0, (f32)extent.y(), 0, 1)});
    cmd.setScissor(0, {rhi::Rect2D(0, 0, extent.x(), extent.y())});

    

    // Render loop
    auto heap = sub<rhi::DeviceManager>().descriptor_heap();
    auto sampler_heap = sub<rhi::DeviceManager>().sampler_heap();
    cmd.bindDescriptorHeap(heap, sampler_heap, heap->size(), sampler_heap->size());

    rhi::Pipeline *last_pipeline{};
    rhi::Buffer   *last_buffer{};
    rhi::Image2   *last_image{};

    u64 cam_addr = cam_ubo ? cam_ubo->address() : 0;
    u64 lig_addr = lig_ubo ? lig_ubo->address() : 0;


    auto view = m_registry.view<ecs::Transform, ecs::Mesh>();
    for (auto entity: view) {
      auto &transform = view.get<ecs::Transform>(entity);
      auto &renderable = view.get<ecs::Mesh>(entity);

      if (!renderable.mesh)
        continue;

      auto mesh = renderable.mesh;

      
      u32 i{};
      for (auto &subsur: mesh->offs()) {
        auto material = renderable.mesh->material()[renderable.mesh->map()[i]];
        auto desc = material->desc(target);

        if (desc->pipeline.get() != last_pipeline) {
          cmd.bindPipeline(desc->pipeline.get());
          last_pipeline = desc->pipeline.get();
        }

        struct PC {
          mat4x4<f32> model;
          u64 vertex_addr;
          u64 camera_addr;
          u64 light_addr;
          u32 texture_id;
        };
        u32 tex_id = material->is_texture() ? material->texture()->id() : 0;
        PC pc = { transform.model, mesh->data()->address(), cam_addr, lig_addr, tex_id };
        cmd.pushConstant(flags(rhi::ShaderStage::Vertex) | rhi::ShaderStage::Pixel, 0, data(&pc, sizeof(pc)));
        
        cmd.draw({subsur.off(), subsur.size()}, {0, 1});
        i++;
      }

    }


    auto views = m_registry.view<ecs::Transform, ecs::MultiMesh>();
    for (auto entity: views) {
      auto &transform = views.get<ecs::Transform>(entity);
      auto &renderable = views.get<ecs::MultiMesh>(entity);

      if (!renderable.mesh)
        continue;

      auto mesh = renderable.mesh;

      
      u32 i{};
      for (auto &subsur: mesh->offs()) {
        auto material = renderable.mesh->material()[renderable.mesh->map()[i]];
        auto desc = material->desc(target);

        if (desc->pipeline.get() != last_pipeline) {
          cmd.bindPipeline(desc->pipeline.get());
          last_pipeline = desc->pipeline.get();
        }

        struct PC_Multi {
          mat4x4<f32> model;
          u64 vertex_addr;
          u64 camera_addr;
          u64 light_addr;
          u64 instance_addr;
          u32 texture_id;
        };
        u32 tex_id = material->is_texture() ? material->texture()->id() : 0;
        PC_Multi pc = { transform.model, mesh->data()->address(), cam_addr, lig_addr, renderable.instances->address(), tex_id };
        cmd.pushConstant(flags(rhi::ShaderStage::Vertex) | rhi::ShaderStage::Pixel, 0, data(&pc, sizeof(pc)));
        
        cmd.draw({subsur.off(), subsur.size()}, {0, renderable.active_count});
        i++;
      }

    }

  }


  fun Core::draw() -> void {
    auto &dev = sub<rhi::DeviceManager>();
    auto &ren = sub<manager::RenderManager>();
    auto &disp = sub<manager::WindowManager>();


    auto &cmd = ren.begin_frame();

    void* wait_sem = ren.get_image_available_sem();

    u32 image_index = disp.acquire_next_image(wait_sem);
    auto &target = disp.get_render_target(image_index);

    ren.begin_pass(cmd, target, {0.1f, 0.1f, 0.1f, 1.0f});

    /* paint */ paint(cmd, target);

    ren.end_pass(cmd, target);

    void* signal_sem = disp.getRenderFinishedSemaphore(image_index);
    ren.end_frame(cmd, wait_sem, signal_sem);

    disp.present(signal_sem, image_index);
  }
  
}
