/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "drivers/vulkan/VKdriver.hh"
#include "drivers/vulkan/manager/VKalloc_manager.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "drivers/vulkan/manager/VKdevice_manager.hh"
#include "mochi/debug/debug.hh"
#include "drivers/vulkan/VKbuffer.hh"
#include <vulkan/vulkan.hpp>
#include <vulkan/vulkan_core.h>
#include "vk_mem_alloc.h"



namespace mochi::rhi::vulkan
{

  VK_Buffer::VK_Buffer(rhi::DeviceManager &device, VmaAllocator vma_allocator, VkBuffer buffer, VmaAllocation allocation, void* mapped, u64 size)
    : m_device(device)
    , m_vma_allocator(vma_allocator)
    , m_buffer(buffer)
    , m_allocation(allocation)
    , m_mapped(mapped)
    , m_size(size)
  {
  }

  VK_Buffer::~VK_Buffer() {
    if (m_vma_allocator)
      vmaDestroyBuffer(m_vma_allocator,m_buffer,m_allocation);
  }


  fun VK_Buffer::map() -> void {
    #ifdef _mochi_debug_validator
      if (m_mapped) [[unlikely]]
        debug::debug(Module, debug::MsgType::Error, "resource already mapped");
    #endif

    vmaMapMemory(m_vma_allocator, m_allocation, &m_mapped);
  }
  
  fun VK_Buffer::unmap() -> void {
    #ifdef _mochi_debug_validator
      if (!m_mapped) [[unlikely]]
        debug::debug(Module, debug::MsgType::Error, "resource not mapped");
    #endif
    
    vmaUnmapMemory(m_vma_allocator, m_allocation);
  }

  fun VK_Buffer::flush(offs off) -> void {
    #ifdef _mochi_debug_validator
      if (!m_mapped) [[unlikely]]
        debug::debug(Module, debug::MsgType::Error, "resource not mapped");
    #endif
    
    vmaFlushAllocation(m_vma_allocator, m_allocation, off.off(), off.size());
  }


  fun VK_Buffer::address() const -> u64 {
    vk::BufferDeviceAddressInfo addr_info{};
    addr_info.sType = vk::StructureType::eBufferDeviceAddressInfo;
    addr_info.buffer = static_cast<VkBuffer>(m_buffer);
    
    return static_cast<VK_DeviceManager&>(m_device).get().getBufferAddress(addr_info);
  }

}
