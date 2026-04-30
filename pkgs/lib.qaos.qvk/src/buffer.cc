/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "qvk/buffer.hh"
#include "qvk/device.hh"
#include "qvk/memory.hh"
#include "vulkan/vulkan.hpp"



namespace qvk
{

  buffer::buffer(qvk::memory &memory, qvk::device &device, vk::DeviceSize size, vk::BufferUsageFlags usage, vk::MemoryPropertyFlags properties)
    : m_buffer(Nil), m_memory(Nil), m_size(size)
  {
    // Create Buffer
    vk::BufferCreateInfo buffer_info(
      {},
      size,
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
    m_mapped = m_memory.mapMemory(0, size, {});
  }


  buffer::~buffer()
  {
    m_memory.unmapMemory();
  }

}
