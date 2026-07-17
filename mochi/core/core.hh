/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/manager/window_manager.hh"
#include "mochi/manager/render_manager.hh"
#include "mochi/manager/scene_manager.hh"
#include "mochi/manager/material_manager.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/rhi/manager/alloc_manager.hh"
#include "mochi/rhi/manager/transfer_manager.hh"
#include "mochi/rhi/manager/shader_manager.hh"
#include <cassert>
#include <functional>
#include "mochi/rhi/command.hh"
#include "entt/entt.hpp"



namespace mochi
{

  struct Core {
    public:
      explicit Core();
      ~Core();

    
    // Properties
    private:
      entt::registry m_registry;
      std::function<fun (f32 dt) -> void> m_idle;

    public:
      fun& registry() { return m_registry; }
      fun& idle() { return m_idle; }
      

    // Sub Module
    private:
      std::tuple<
        uptr<rhi::DeviceManager>,
        uptr<rhi::AllocManager>,
        uptr<rhi::TransferManager>,
        uptr<rhi::ShaderManager>,
        uptr<rhi::PipelineManager>,
        uptr<manager::MaterialManager>,
        uptr<manager::SceneManager>,
        uptr<manager::WindowManager>,
        uptr<manager::RenderManager>
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
      fun paint(rhi::Command &cmd, rhi::RenderTarget &target) -> void;
      fun draw() -> void;
  };

}
