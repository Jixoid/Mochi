/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/image.hh"
#include "mochi/rhi/vtype.hh"
#include "mochi/types.hh"
#include <vulkan/vulkan_core.h>



namespace mochi::rhi
{

  template<>
  struct info<buffer>
  {
    private:
      explicit inline info<buffer>(u64 stride, std::vector<vt> items): m_stride(stride), m_items(items) {}

    public:
      static inline fun make(u64 stride, std::vector<vt> items) {
        return make_sptr(new info<buffer>(stride, items));
      }


    private:
      std::vector<vt> m_items;
      u64 m_stride;

    public:
      inline fun items() { return m_items; }
      inline fun stride() { return m_stride; }
  };




  struct buffer
  {
    private:
      explicit buffer(module::device &device, module::memory &memory, sptr<info<buffer>> info, u64 count, BufferUsageFlags usage, BufferCreateFlags create, BufferLocation location);
      
    public:
      ~buffer();
      
      static fun make(module::device &device, module::memory &memory, sptr<info<buffer>> info, u64 count, BufferUsageFlags usage, BufferCreateFlags create, BufferLocation location, std::function<void (void*)> data = nil) -> sptr<buffer>;
    

    protected:
      module::device &m_device;
      sptr<info<buffer>> m_info{};
      u64   m_size{};
      void* m_mapped{};
      VmaAllocator m_allocator{nil};
      VkBuffer m_buffer{nil};
      VmaAllocation m_allocation{nil};
      VmaAllocationInfo m_alloc_info{};

    public:
      inline fun info() { return m_info.get(); }
      inline fun size() const { return m_size; }
      inline fun mapped() const { return m_mapped; }
      inline fun get() const { return m_buffer; }
      fun address() const -> VkDeviceAddress;
  };

}
