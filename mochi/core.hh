/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/module/display.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/rhi/manager/material_manager.hh"
#include "mochi/rhi/manager/alloc_manager.hh"
#include "mochi/rhi/manager/transfer_manager.hh"
#include "mochi/rhi/manager/shader_manager.hh"
#include "mochi/module/renderer.hh"
#include "mochi/module/memory.hh"
#include <cassert>
#include <functional>
#include <tuple>
#include "mochi/rhi/command.hh"
#include "entt/entt.hpp"



namespace mochi
{

  struct core {
    public:
      explicit core(
        std::function<i32 ()> GpuPicker,
        std::function<void (f32 dt)> Idle
      );

      ~core();


    // Properties
    private:
      entt::registry m_registry;

    public:
      fun& registry() { return m_registry; }
      
    private:
      std::function<void (f32 dt)> m_idle;


    // Sub Module
    private:
      std::tuple<
        uptr<rhi::DeviceManager>,
        uptr<rhi::AllocManager>,
        uptr<rhi::TransferManager>,
        uptr<rhi::ShaderManager>,
        uptr<rhi::MaterialManager>,
        uptr<module::memory>,
        uptr<module::display>,
        uptr<module::renderer>
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
      fun paint(rhi::Command &cmd, rhi::render_target &target) -> void;
      fun draw() -> void;
  };

}
