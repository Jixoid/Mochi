/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/types.hh"
#include <vulkan/vulkan_core.h>
#include <vulkan/vulkan_raii.hpp>
#include <unordered_map>
#include <vector>


namespace mochi::rhi::vulkan
{

  struct VK_Queue: rhi::Queue {
    public:
      explicit VK_Queue() {}

    private:
      vk::raii::Queue m_queue{nil};

    public:
      fun& get() { return m_queue; }
  };



  struct VK_DeviceManager: rhi::DeviceManager {
    public:
      explicit VK_DeviceManager(std::string_view appName, std::array<u16, 4> appVer);

      fun waitIdle() -> void override { vk_device.waitIdle(); }


    private:
      static fun f_is_suitable(vk::raii::PhysicalDevice) -> bool;
      

    private:
      vk::raii::Context vk_ctx;
      vk::raii::Instance vk_inst;

      vk::raii::PhysicalDevice vk_phys_dev;
      vk::raii::Device vk_device;

      vk::raii::CommandPool m_mainPool; // Has Graphic & Compute
      
      std::unordered_map<u32, std::vector<vk::raii::Queue>> m_active_queues;
      
    public:
      fun& active_queue(u32 family) { return m_active_queues[family][0]; }
    public:
      fun& ctx() { return vk_ctx; }
      fun& inst() { return vk_inst; }
      
      fun& phys_dev() { return vk_phys_dev; }
      fun& get() { return vk_device; }

      fun getMainBuffer(u32 count) -> vk::raii::CommandBuffers;
  };

}
