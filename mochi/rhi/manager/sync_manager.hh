/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/types.hh"


namespace mochi::rhi
{
  // External
  extern "C" fun MochiRHI_MakeSyncManager(rhi::DeviceManager &dmng, u32 max_frames_in_flight) -> SyncManager*;


  // Interface
  struct SyncManager: noncopy {
    protected:
      SyncManager(rhi::DeviceManager &dmng, u32 max_frames_in_flight)
        : m_dmng(dmng)
        , m_max_frames_in_flight(max_frames_in_flight)
      {}

    public:
      virtual ~SyncManager() = default;

      static fun make(rhi::DeviceManager &device, u32 max_frames_in_flight = 2) {
        return make_sptr(MochiRHI_MakeSyncManager(device, max_frames_in_flight));
      }

    protected:
      rhi::DeviceManager &m_dmng;
      u32 m_max_frames_in_flight;

    public:
      // CPU-GPU kare senkronizasyonu
      virtual fun beginFrame() -> void = 0;
      virtual fun endFrame() -> void = 0;

      virtual fun currentFrameIndex() const -> u32 = 0;
      virtual fun maxFramesInFlight() const -> u32 = 0;

      // Swapchain ve Queue Submit için ham senkronizasyon nesnelerini void* (veya vk::raii) olarak döner
      virtual fun activeInFlightFence() -> void* = 0;
      virtual fun activeImageAvailableSemaphore() -> void* = 0;
  };
  
}
