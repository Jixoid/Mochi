/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "drivers/vulkan/VKimage.hh"
#include <vulkan/vulkan_raii.hpp>
#include "mochi/rhi/manager/alloc_manager.hh"
#include "vk_mem_alloc.h"



namespace mochi::rhi::vulkan
{

  struct VK_AllocManager final: public rhi::AllocManager {
    public:
      explicit VK_AllocManager(rhi::DeviceManager &device);
      ~VK_AllocManager();

    private:
      VmaAllocator vma_allocator{nil};

    public:
      #ifdef _mochi_driver_debug_statistics
        class {
          friend struct VK_AllocManager;
          friend struct VK_Buffer;
          friend struct VK_Image2;

          u64 m_buffer{}, m_image2{};
        }
        m_lives;
      
      #endif

    public:
      fun& get() { return vma_allocator; }
      
    public:
      fun allocBuffer(
        u64 size, BufferUsageFlags usage, AllocationCreateFlags create, AllocationLocation location
      ) -> sptr<Buffer> override;
      
      fun allocImage2(
        extent<2,u32> ext, Format format, ImageUsageFlags usage, ImageTiling tiling,
        AllocationCreateFlags create, AllocationLocation location
      ) -> sptr<Image2> override;

      fun allocSampler2(
        extent<3, SamplerAddressMode> addressMode, SamplerFilter magFilter, SamplerFilter minFilter
      ) -> sptr<Sampler2> override;

    public:
      // This driver does not yet supported statistics
      fun support_statistics() -> bool override { return false; };

      fun stat_total_vram() -> u64 override { return 0; };
      fun stat_cache_vram() -> u64 override { return 0; };
      fun stat_used_vram() -> u64 override { return 0; };
      
      fun stat_live_buffer() -> u64 override {
        #ifdef _mochi_driver_debug_statistics
          return m_lives.m_buffer;
        #else
          return 0;
        #endif
      };
      fun stat_live_image2() -> u64 override {
        #ifdef _mochi_driver_debug_statistics
          return m_lives.m_image2;
        #else
          return 0;
        #endif
      };

  };

}
