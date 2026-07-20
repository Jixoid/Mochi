/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "mochi/basis.hh"
#include "mochi/ecs/multi_mesh.hh"
#include "mochi/ecs/point_light.hh"
#include "mochi/ecs/camera.hh"
#include "mochi/ecs/transform.hh"
#include "mochi/ecs/mesh.hh"
#include "mochi/types.hh"
#include "mochi/systems/rendering_system.hh"
#include "mochi/systems/display_system.hh"
#include "mochi/rhi/manager/command_manager.hh"
#include <cassert>



namespace mochi::sys
{

  RenderingSystem::RenderingSystem(rhi::Device &device, rhi::Allocator &alloc, rhi::CommandManager &cmng, rhi::Synchronizer &sync)
    : m_device(device), m_alloc(alloc), m_cmng(cmng), m_sync(sync)
  {
    for (u32 i = 0; i < MAX_FRAMES_IN_FLIGHT; i++)
      m_cmd_buffers.push_back(m_cmng.allocateGraphicsCommand());
  }

  RenderingSystem::~RenderingSystem() {
    m_light_ubo.reset();
    m_camera_ubo.reset();
  }



  fun RenderingSystem::draw(rhi::Command &cmd, rhi::RenderTarget &target, asset::Scene *scene, sys::DisplaySystem &disp) -> void {
    auto &regs = scene->regs();


    // Update Camera
    auto cameras = regs.view<ecs::Transform, ecs::Camera>();
    size_t cam_count = std::distance(cameras.begin(), cameras.end());
    prepare_camera_ubo(std::max<size_t>(1, cam_count));
    
    l_write_camera_ubo: {
      auto* cam_data = (ecs::camera_t*)m_camera_ubo->mapped();
      u32 idx = 0;
      for (auto entity: cameras) {
        auto &cam = cameras.get<ecs::Camera>(entity);
        cam_data[idx].view = cam.view;
        cam_data[idx].proj = cam.proj;
        idx++;
      }
    }


    // Update Light
    auto lights = regs.view<ecs::Transform, ecs::PointLight>();
    size_t lig_count = std::distance(lights.begin(), lights.end());
    prepare_light_ubo(std::max<size_t>(1, lig_count+1));

    l_write_light_ubo: {
      auto* lig_data = (ecs::point_light_t*)m_light_ubo->mapped();
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


    // Update Viewport
    auto extent = disp.extent();
    cmd.setViewport(0, {rhi::Viewport(0, (f32)extent.x(), 0, (f32)extent.y(), 0, 1)});
    cmd.setScissor(0, {rhi::Rect2D(0, 0, extent.x(), extent.y())});


    // Render loop
    auto heap = m_device.descriptor_heap();
    auto sampler_heap = m_device.sampler_heap();
    cmd.bindDescriptorHeap(heap, sampler_heap, heap->size(), sampler_heap->size());

    rhi::Pipeline *last_pipeline{};
    rhi::Buffer   *last_buffer{};
    rhi::Image2   *last_image{};

    u64 cam_addr = m_camera_ubo ? m_camera_ubo->address() : 0;
    u64 lig_addr = m_light_ubo ? m_light_ubo->address() : 0;


    auto view = regs.view<ecs::Transform, ecs::Mesh>();
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
          mat4x3<f32> model;
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


    auto views = regs.view<ecs::Transform, ecs::MultiMesh>();
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
          mat4x3<f32> model;
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



  fun RenderingSystem::begin_frame() -> rhi::Command& {
    m_sync.beginFrame();

    auto &cmd = *m_cmd_buffers[current_frame()];
    
    cmd.begin();
    
    if (m_device.descriptor_heap()) {
      cmd.bindDescriptorHeap(m_device.descriptor_heap(), m_device.sampler_heap(), m_device.descriptor_heap()->size(), m_device.sampler_heap()->size());
    }

    return cmd;
  }

  fun RenderingSystem::end_frame(rhi::Command &cmd, void *wait_sem, void *signal_sem) -> void {
    cmd.end();
    
    m_cmng.submitWithSync(&cmd, wait_sem, signal_sem, m_sync.activeInFlightFence());

    m_sync.endFrame();
  }


  fun RenderingSystem::begin_pass(rhi::Command &cmd, const rhi::RenderTarget &target, const std::array<f32,4> &clear_color) -> void {
    cmd.beginRendering(target, clear_color);
  }

  fun RenderingSystem::end_pass(rhi::Command &cmd, const rhi::RenderTarget &target) -> void {
    cmd.endRendering(target);
  }



  fun RenderingSystem::prepare_light_ubo(u64 required_count) -> void {
    if (!m_light_ubo || (m_light_ubo->size() / sizeof(ecs::point_light_t)) < required_count)
      m_light_ubo = m_alloc.allocBuffer(
        sizeof(ecs::point_light_t) * std::max<u64>(100, required_count),
        rhi::BufferUsage::DeviceAddress, 
        rhi::AllocationCreate::HostSequentialWrite | rhi::AllocationCreate::Mapped,
        rhi::AllocationLocation::PreferHost
      );
  }

  fun RenderingSystem::prepare_camera_ubo(u64 required_count) -> void {
    if (!m_camera_ubo || (m_camera_ubo->size() / sizeof(ecs::camera_t)) < required_count)
      m_camera_ubo = m_alloc.allocBuffer(
        sizeof(ecs::camera_t) * std::max<u64>(100, required_count),
        rhi::BufferUsage::DeviceAddress, 
        rhi::AllocationCreate::HostSequentialWrite | rhi::AllocationCreate::Mapped,
        rhi::AllocationLocation::PreferHost
      );
  }

}
