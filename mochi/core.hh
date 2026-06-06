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
#include "mochi/module/resource.hh"
#include "mochi/module/memory.hh"
#include "mochi/module/renderer.hh"
#include "mochi/rhi/device.hh"
#include <cassert>
#include <functional>
#include <tuple>
#include <vulkan/vulkan_raii.hpp>
#include "mochi/rhi/cmd.hh"
#include "src/entt/entt.hpp"



namespace mochi
{

  /** @brief The central class that initializes and manages all mochi subsystems. */
  struct core
  {
    public:
      /**
       * @brief Construct the core mochi engine instance.
       * @param GpuPicker A callback to select the preferred Vulkan physical device.
       * @param Idle A callback executed every frame, typically used for game logic updates.
       */
      explicit core(
        std::function<i32 (const vk::raii::PhysicalDevices&, rhi::PhysicalDeviceSuitable)> GpuPicker,
        std::function<void (f32 dt)> Idle
      );

      /** @brief Destructor to safely terminate the engine and wait for device idle. */
      ~core();



    // Properties
    private:
      entt::registry m_registry;

    public:
      /** @brief Access the active ECS registry. */
      inline fun& registry() { return m_registry; }
      
    private:
      std::function<void (f32 dt)> m_idle;


    // Sub Module
    private:
      std::tuple<
        uptr<rhi::device>,
        uptr<module::resource>,
        uptr<module::memory>,
        uptr<module::display>,
        uptr<module::renderer>
      > m_modules;

    public:
      /**
       * @brief Get a reference to a specific sub-module.
       * @tparam T The requested sub-module type.
       * @return Reference to the sub-module.
       */
      template <typename T>
      inline fun& sub() { return *std::get<uptr<T>>(m_modules); } 


    // Functions
    public:
      /** @brief Start the main application loop. */
      fun run() -> void;

    private:
      fun paint(rhi::cmd &cmd, rhi::render_target &target) -> void;
      fun draw() -> void;
  };

}
