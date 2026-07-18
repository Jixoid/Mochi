/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/rhi/manager/device_manager.hh"
#include <vulkan/vulkan_raii.hpp>
#include "mochi/rhi/sampler.hh"
#include "vk_mem_alloc.h"



namespace mochi::rhi::vulkan
{

  struct VK_Sampler2 final: public rhi::Sampler2 {
    public:
      explicit VK_Sampler2(rhi::mng::DeviceManager &device, vk::raii::Sampler sampler): m_device(device), m_sampler(std::move(sampler)) {}

    private:
      rhi::mng::DeviceManager &m_device;
      vk::raii::Sampler m_sampler;

    public:
      fun get() -> VkSampler { return *m_sampler; }  
  };

}
