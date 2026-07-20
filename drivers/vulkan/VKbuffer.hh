/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "drivers/vulkan/VKconvert.hh"
#include "mochi/rhi/manager/device.hh"
#include <vulkan/vulkan.hpp>
#include "vk_mem_alloc.h"



namespace mochi::rhi::vulkan
{

  struct VK_Buffer final: public rhi::Buffer {
    public:
      explicit VK_Buffer(rhi::Device &device, VmaAllocator vma_allocator, VkBuffer buffer, VmaAllocation allocation, void* mapped, u64 size);
      ~VK_Buffer();

    private:
      rhi::Device &m_device;
      VmaAllocator m_vma_allocator{nil};
      VkBuffer m_buffer{nil};
      VmaAllocation m_allocation{nil};
      void* m_mapped{};
      u64 m_size{};
    
    public:
      fun size() const -> u64 override { return m_size; };

      fun map() -> void override;
      fun unmap() -> void override;
      fun flush(offs) -> void override;

      fun mapped() const -> void* override { return m_mapped; };
      fun address() const -> u64 override;

    public:
      fun get() { return m_buffer; }
  };

}
