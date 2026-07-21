/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/manager/device.hh"
#include "mochi/rhi/utility/pipeline_cache_utility.hh"
#include "mochi/rhi/utility/shader_cache_utility.hh"
#include "mochi/rhi/utility/shader_compile_utility.hh"
#include "mochi/systems/plugin_system.hh"
#include "mochi/systems/display_system.hh"
#include "mochi/systems/rendering_system.hh"
#include "mochi/systems/scene_system.hh"
#include "mochi/systems/audio_system.hh"
#include <cassert>
#include <functional>
#include "entt/entt.hpp"



namespace mochi
{

  struct Engine {
    public:
      explicit Engine();
      ~Engine();

    
    // Properties
    private:
      std::function<fun (f32 dt) -> void> m_idle;

    public:
      fun& idle() { return m_idle; }
      

    // Sub Module
    private:
      std::tuple<
        uptr<rhi::Device>,
        uptr<rhi::Allocator>,
        uptr<rhi::Uploader>,
        uptr<rhi::CommandManager>,
        uptr<rhi::Synchronizer>,

        uptr<sys::RenderingSystem>,
        uptr<sys::DisplaySystem>,
        uptr<sys::SceneSystem>,
        uptr<sys::PluginSystem>,
        uptr<sys::AudioSystem>,
        
        uptr<rhi::PipelineCacheUtility>,
        uptr<rhi::ShaderCacheUtility>,
        uptr<rhi::ShaderCompileUtility>,
        uptr<utility::MaterialUtility>
      > m_modules;

    public:
      /// @brief Get a reference to a specific sub-module
      template <typename T>
      fun& sub() { return *std::get<uptr<T>>(m_modules); } 


    // Functions
    public:
      /// @brief Start the main application loop
      fun run() -> void;

    private:
      fun draw() -> void;
  };

}
