/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "qvk/bridge.hh"
#include "qvk/window.hh"
#include "qvk/device.hh"
#include "qvk/memory.hh"
#include "qvk/swapchain.hh"
#include "qvk/renderer.hh"
#include "qvk/meta.hh"
#include <cassert>
#include <functional>
#include <vulkan/vulkan_raii.hpp>



namespace qvk
{

  struct engine
  {
    public:
      explicit engine(
        std::function<vk::raii::PhysicalDevice (vk::raii::PhysicalDevices)> GpuPicker,
        std::function<void ()> Idle
      );

    public:
      ~engine();



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
        requires std::derived_from<T, qvk::window>
      inline fun& sub() { return m_window; }

      template <typename T>
        requires std::derived_from<T, qvk::device>
      inline fun& sub() { return m_device; }

      template <typename T>
        requires std::derived_from<T, qvk::memory>
      inline fun& sub() { return m_memory; }

      template <typename T>
        requires std::derived_from<T, qvk::swapchain>
      inline fun& sub() { return m_swapchain; }

      template <typename T>
        requires std::derived_from<T, qvk::meta>
      inline fun& sub() { return m_meta; }


    // Functions
    public:
      fun run() -> void;

    private:
      fun draw() -> void;
  };

}
