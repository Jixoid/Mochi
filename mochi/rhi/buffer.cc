/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/rhi/convert.hh"
#include "mochi/module/memory.hh"
#include "mochi/except.hh"
#include "mochi/types.hh"



namespace mochi::rhi
{

  buffer::buffer(module::device &device, module::memory &memory, sptr<rhi::info<buffer>> info, u64 count, BufferUsageFlags usage, BufferCreateFlags create)
    : m_info(info)
    , m_size(info->stride() * count)
    , m_buffer(nil)
    , m_allocator(memory.allocator())
  {
    VmaAllocationCreateInfo alloc_info{};
    alloc_info.usage = VmaMemoryUsage::VMA_MEMORY_USAGE_AUTO;
    alloc_info.flags = VKConvert<BufferCreateFlags>(create);

    VkBufferCreateInfo buffer_create_info{};
    buffer_create_info.sType = VkStructureType::VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO;
    buffer_create_info.size = m_size;
    buffer_create_info.usage = static_cast<VkBufferUsageFlags>(VKConvert<BufferUsageFlags>(usage));
    buffer_create_info.sharingMode = VkSharingMode::VK_SHARING_MODE_EXCLUSIVE;

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

    
    if (create & BufferCreate::Mapped)
      m_mapped = m_alloc_info.pMappedData;
  }

  buffer::~buffer()
  {
    if (m_buffer) vmaDestroyBuffer(m_allocator, m_buffer, m_allocation);
  }

}
