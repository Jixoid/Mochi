/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "qvk/entity/buffer.hh"
#include "qvk/module/device.hh"
#include "qvk/module/memory.hh"
#include "qvk/core.hh"
#include "vulkan/vulkan.hpp"



namespace qvk
{

  fun info<buffer>::make(core &core, u64 stride, std::vector<qvk::gt> items) -> info<buffer>*
  {
    auto obj = new info<buffer>(stride, items);
    
    core.sub<memory>().push<info<buffer>>(obj);
    return obj;
  }


  

  buffer::buffer(device &device, qvk::memory &memory, qvk::info<buffer> *info, u64 count, vk::BufferUsageFlags usage, vk::MemoryPropertyFlags properties)
    : m_info(info)
    , m_buffer(Nil)
    , m_memory(Nil)
    , m_size(info->stride() * count)
  {
    // Create Buffer
    vk::BufferCreateInfo buffer_info(
      {},
      m_size,
      usage,
      vk::SharingMode::eExclusive
    );
    m_buffer = vk::raii::Buffer(device.vdevice(), buffer_info);

    vk::MemoryRequirements mem_reqs = m_buffer.getMemoryRequirements();


    // Find Suitable Memory
    vk::MemoryAllocateInfo alloc_info(
      mem_reqs.size,
      memory.find_memory_type(mem_reqs.memoryTypeBits, properties)
    );
    m_memory = vk::raii::DeviceMemory(device.vdevice(), alloc_info);


    // Bind Buffer
    m_buffer.bindMemory(*m_memory, 0);

    // Maping
    m_mapped = m_memory.mapMemory(0, m_size, {});
  }


  buffer::~buffer()
  {
    m_memory.unmapMemory();
  }

}
