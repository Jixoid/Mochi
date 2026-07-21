/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/core/engine.hh"
#include "mochi/rhi/manager/device.hh"
#include "mochi/rhi/manager/allocator.hh"
#include "mochi/rhi/manager/uploader.hh"
#include "mochi/rhi/manager/command_manager.hh"
#include "mochi/rhi/utility/pipeline_cache_utility.hh"
#include "mochi/utility/material_utility.hh"
#include "mochi/systems/display_system.hh"
#include "mochi/systems/plugin_system.hh"
#include "mochi/systems/rendering_system.hh"
#include "mochi/systems/scene_system.hh"
#include "mochi/systems/audio_system.hh"
#include "mochi/types.hh"
#include <chrono>



namespace mochi
{

  Engine::Engine() {
    // RHI
    auto device = rhi::Device::make("Mochi", {1,0,0,0});
    auto alloc = rhi::Allocator::make(*device);
    device->initDescriptorHeap(*alloc);
    
    auto upload = rhi::Uploader::make(*device);

    auto cmng = rhi::CommandManager::make(*device);
    auto sync = rhi::Synchronizer::make(*device);


    // System
    auto rendering = make_uptr(new sys::RenderingSystem(*device, *alloc, *cmng, *sync));
    auto smng = make_uptr(new sys::SceneSystem());
    auto display = make_uptr(new sys::DisplaySystem(*device, *smng, "Mochi Engine", 800,600));
    auto plugin = make_uptr(new sys::PluginSystem(*this));
    auto audio = make_uptr(new sys::AudioSystem());

    // Utility
    auto picu = rhi::PipelineCacheUtility::make(*device);
    auto sacu = rhi::ShaderCacheUtility::make(*device);
    auto shcu = rhi::ShaderCompileUtility::make(*device);

    auto umat = make_uptr(new utility::MaterialUtility(*device, *shcu, *sacu, *picu));


    // Finish
    m_modules = decltype(m_modules)(
      std::move(device),
      std::move(alloc),
      std::move(upload),
      std::move(cmng),
      std::move(sync),

      std::move(rendering),
      std::move(display),
      std::move(smng),
      std::move(plugin),
      std::move(audio),
      
      std::move(picu),
      std::move(sacu),
      std::move(shcu),
      std::move(umat)
    );
  }

  Engine::~Engine() {
    auto& device = std::get<uptr<rhi::Device>>(m_modules);
    if (device) {
      device->waitIdle();
      device->finiDescriptorHeap();
    }

    std::get<uptr<utility::MaterialUtility>>(m_modules).reset();
    std::get<uptr<rhi::ShaderCompileUtility>>(m_modules).reset();
    std::get<uptr<rhi::ShaderCacheUtility>>(m_modules).reset();
    std::get<uptr<rhi::PipelineCacheUtility>>(m_modules).reset();
    
    std::get<uptr<sys::AudioSystem>>(m_modules).reset();
    std::get<uptr<sys::PluginSystem>>(m_modules).reset();
    std::get<uptr<sys::SceneSystem>>(m_modules).reset();
    std::get<uptr<sys::DisplaySystem>>(m_modules).reset();
    std::get<uptr<sys::RenderingSystem>>(m_modules).reset();

    std::get<uptr<rhi::Synchronizer>>(m_modules).reset();
    std::get<uptr<rhi::CommandManager>>(m_modules).reset();
    std::get<uptr<rhi::Uploader>>(m_modules).reset();
    std::get<uptr<rhi::Allocator>>(m_modules).reset();
    
    std::get<uptr<rhi::Device>>(m_modules).reset();
  }



  fun Engine::run() -> void {
    auto last_time = std::chrono::high_resolution_clock::now();
    
    while (sub<sys::DisplaySystem>().proc_events()) {
      draw();

      auto current_time = std::chrono::high_resolution_clock::now();
      f32 dt = std::chrono::duration<f32, std::chrono::seconds::period>(current_time-last_time).count();
      last_time = current_time;
        
      m_idle(dt);
      sub<sys::PluginSystem>().dispatch(HookThing::Update);
    }
  }


  fun Engine::draw() -> void {
    auto &rend = sub<sys::RenderingSystem>();
    auto &disp = sub<sys::DisplaySystem>();
    auto &smng = sub<sys::SceneSystem>();

    if (disp.resized())
      disp.recreate_swapchain();

    auto &cmd = rend.begin_frame();

    void* wait_sem = rend.get_image_available_sem();

    u32 image_index = disp.acquire_next_image(wait_sem);
    auto &target = disp.get_render_target(image_index);

    rend.begin_pass(cmd, target, {0.1f, 0.1f, 0.1f, 1.0f});

    rend.draw(cmd, target, smng.getActiveScene().get(), disp);

    rend.end_pass(cmd, target);

    void* signal_sem = disp.getRenderFinishedSemaphore(image_index);
    rend.end_frame(cmd, wait_sem, signal_sem);

    disp.present(signal_sem, image_index);
  }
  
}
