/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/types.hh"
#include <vulkan/vulkan_raii.hpp>
#include "vk_mem_alloc.h"



namespace mochi::rhi
{

  struct image2
  {
    private:
      explicit image2(rhi::device &device, module::memory &memory, u32 width, u32 height, void *ptr);
      
    public:
      ~image2();

      static inline fun make(rhi::device &device, module::memory &memory, u32 width, u32 height, void *ptr) -> sptr<image2> {
        return make_sptr(new image2(device, memory, width, height, ptr));
      }


    private:
      u32 m_width, m_height;
      VmaAllocator m_allocator{nil};
      VkImage m_image{VK_NULL_HANDLE};
      VmaAllocation m_allocation{nil};
      vk::raii::ImageView m_view{nil};
      vk::raii::Sampler m_sampler{nil};

    public:
      inline fun  width() const { return m_width; }
      inline fun  height() const { return m_height; }
      inline fun& image() { return m_image; }
      inline fun& view() { return m_view; }
      inline fun& sampler() { return m_sampler; }
  };

}
