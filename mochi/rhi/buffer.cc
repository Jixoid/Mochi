/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/rhi/buffer.hh"
#include "mochi/module/device.hh"
#include "mochi/module/memory.hh"
#include "mochi/except.hh"
#include "vulkan/vulkan.hpp"
#include "vk_mem_alloc.h"
#include "vk_mem_alloc.h"



namespace mochi::rhi
{

  buffer::buffer(module::device &device, module::memory &memory, rhi::info<buffer> *info, u64 count, vk::BufferUsageFlags usage, const VmaAllocationCreateInfo &alloc_info)
    : m_info(info)
    , m_buffer(nil)
    , m_size(info->stride() * count)
    , m_allocator(memory.allocator())
  {
    VkBufferCreateInfo buffer_create_info = {};
    buffer_create_info.sType = VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO;
    buffer_create_info.size = m_size;
    buffer_create_info.usage = static_cast<VkBufferUsageFlags>(usage);
    buffer_create_info.sharingMode = VK_SHARING_MODE_EXCLUSIVE;

    auto res = vmaCreateBuffer(
      m_allocator, 
      &buffer_create_info, 
      &alloc_info, 
      &m_buffer, 
      &m_allocation, 
      &m_alloc_info
    );

    if (res != VK_SUCCESS)
      throw mochi::rhi_error("Failed to create VMA Buffer! Error code: " + std::to_string(res));

    
    if (alloc_info.flags & VMA_ALLOCATION_CREATE_MAPPED_BIT)
      m_mapped = m_alloc_info.pMappedData;
  }


  buffer::~buffer()
  {
    if (m_buffer)
      vmaDestroyBuffer(m_allocator, m_buffer, m_allocation);
  }

}
