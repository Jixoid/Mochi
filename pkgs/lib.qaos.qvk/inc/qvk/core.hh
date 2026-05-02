/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "qvk/module/bridge.hh"
#include "qvk/module/window.hh"
#include "qvk/module/device.hh"
#include "qvk/module/memory.hh"
#include "qvk/module/swapchain.hh"
#include "qvk/module/renderer.hh"
#include "qvk/module/meta.hh"
#include <cassert>
#include <functional>
#include <vulkan/vulkan_raii.hpp>



namespace qvk
{

  struct core
  {
    public:
      explicit core(
        std::function<vk::raii::PhysicalDevice (vk::raii::PhysicalDevices)> GpuPicker,
        std::function<void ()> Idle
      );

    public:
      ~core();



    // Sub Module
    private:
      bridge    m_bridge;
      window    m_window;
      device    m_device;
      memory    m_memory;
      swapchain m_swapchain;
      renderer  m_renderer;
      meta      m_meta;

      std::function<void ()> m_idle;


    public:
      template <typename T>
        requires std::is_same_v<T, qvk::window>
      inline fun& sub() { return m_window; }

      template <typename T>
        requires std::is_same_v<T, qvk::device>
      inline fun& sub() { return m_device; }

      template <typename T>
        requires std::is_same_v<T, qvk::memory>
      inline fun& sub() { return m_memory; }

      template <typename T>
        requires std::is_same_v<T, qvk::swapchain>
      inline fun& sub() { return m_swapchain; }

      template <typename T>
        requires std::is_same_v<T, qvk::meta>
      inline fun& sub() { return m_meta; }


    // Functions
    public:
      fun run() -> void;

    private:
      fun draw() -> void;
  };

}
