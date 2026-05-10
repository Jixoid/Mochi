/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/module/resource.hh"



namespace mochi::module
{

  resource::resource(module::device &device)
    : m_device(device)
  {
    create_pool();
  }
  

  fun resource::create_pool() -> vk::raii::DescriptorPool&
  {
    std::array<vk::DescriptorPoolSize, 2> pool_sizes = {
      vk::DescriptorPoolSize(vk::DescriptorType::eUniformBuffer, 1000),
      vk::DescriptorPoolSize(vk::DescriptorType::eCombinedImageSampler, 1000)
    };

    vk::DescriptorPoolCreateInfo pool_info(
      vk::DescriptorPoolCreateFlagBits::eFreeDescriptorSet, 
      1000, 
      pool_sizes
    );

    m_pools.push_back(vk::raii::DescriptorPool(m_device.vdevice(), pool_info));
    return m_pools.back();
  }



  fun resource::allocate_descriptor_set(vk::DescriptorSetLayout layout) -> vk::raii::DescriptorSet
  {
    vk::DescriptorSetAllocateInfo alloc_info(*m_pools.back(), 1, &layout);
    
    try {
      vk::raii::DescriptorSets sets(m_device.vdevice(), alloc_info);
      return std::move(sets.front());
    }
    catch(const vk::OutOfPoolMemoryError &e) {
      auto &new_pool = create_pool();
      vk::DescriptorSetAllocateInfo new_alloc_info(*new_pool, 1, &layout);
      vk::raii::DescriptorSets sets(m_device.vdevice(), new_alloc_info);
      return std::move(sets.front());
    }
    catch(const vk::FragmentedPoolError &e) {
      auto &new_pool = create_pool();
      vk::DescriptorSetAllocateInfo new_alloc_info(*new_pool, 1, &layout);
      vk::raii::DescriptorSets sets(m_device.vdevice(), new_alloc_info);
      return std::move(sets.front());
    }
  }

}
