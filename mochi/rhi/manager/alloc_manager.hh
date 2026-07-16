/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/math/extent.hh"
#include "mochi/rhi/image.hh"
#include "mochi/rhi/sampler.hh"
#include "mochi/types.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/rhi/rhi.hh"



namespace mochi::rhi
{
  // Enums
  enum struct AllocationCreate: u32 {
    Dedicated           = 0x00000001,
    Mapped              = 0x00000004,
    HostSequentialWrite = 0x00000400,
    HostRandomAccess    = 0x00000800,
    HostTransfer        = 0x00001000,
  };
  using AllocationCreateFlags = flags<AllocationCreate>;


  enum struct AllocationLocation: u32 {
    Auto         = 7,
    PreferDevice = 8,
    PreferHost   = 9,
  };


  // External
  extern "C" fun MochiRHI_MakeAllocManager(rhi::DeviceManager &dmng) -> AllocManager*;


  // Interface
  struct AllocManager: noncopy {
    protected:
      AllocManager(rhi::DeviceManager &dmng): m_dmng(dmng) {}

    public:
      virtual ~AllocManager() = default;
      
      static fun make(rhi::DeviceManager &device) {
        return make_uptr(MochiRHI_MakeAllocManager(device));
      }

    protected:
      rhi::DeviceManager &m_dmng;

    public:
      virtual fun allocBuffer(
        u64 size, BufferUsageFlags usage, AllocationCreateFlags create, AllocationLocation location
      ) -> sptr<Buffer> = 0;
      
      virtual fun allocImage2(
        extent<2,u32> ext, Format format, ImageUsageFlags usage, ImageTiling tiling,
        AllocationCreateFlags create, AllocationLocation location
      ) -> sptr<Image2> = 0;

      virtual fun allocSampler2(
        extent<3, SamplerAddressMode> addressMode, SamplerFilter magFilter, SamplerFilter minFilter 
      ) -> sptr<Sampler2> = 0;

    public:
      // This feature must be enabled with _mochi_driver_debug_statistics
      virtual fun support_statistics() -> bool = 0;

      virtual fun stat_total_vram() -> u64 = 0;
      virtual fun stat_cache_vram() -> u64 = 0;
      virtual fun stat_used_vram() -> u64 = 0;

      virtual fun stat_live_buffer() -> u64 = 0;
      virtual fun stat_live_image2() -> u64 = 0;
  };

}

FlagEnable(mochi::rhi::AllocationCreate)
