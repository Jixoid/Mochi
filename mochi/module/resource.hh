/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/module/device.hh"
#include <vulkan/vulkan_raii.hpp>
#include <vector>



namespace mochi::module
{

  /** @brief Manages resource allocations such as descriptor pools and sets. */
  struct resource
  {
    public:
      /**
       * @brief Initialize a new resource module.
       * @param device The logical device.
       */
      explicit resource(module::device &device);
      

    private:
      module::device &m_device;
      
      std::vector<vk::raii::DescriptorPool> m_pools;

      /** @brief Create a new descriptor pool and add it to the list. */
      fun create_pool() -> vk::raii::DescriptorPool&;

    public:
      /**
       * @brief Allocate a single descriptor set from the available pools.
       * If the active pool is full, it automatically creates a new one.
       * @param layout The layout for the descriptor set.
       * @return The allocated descriptor set.
       */
      fun allocate_descriptor_set(vk::DescriptorSetLayout layout) -> vk::raii::DescriptorSet;
  };
  
}
