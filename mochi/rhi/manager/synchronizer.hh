/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/manager/device.hh"
#include "mochi/types.hh"


namespace mochi::rhi
{
  // External
  extern "C" fun MochiRHI_MakeSynchronizer(rhi::Device &device, u32 max_frames_in_flight) -> Synchronizer*;


  // Interface
  struct Synchronizer: noncopy {
    protected:
      Synchronizer(rhi::Device &device, u32 max_frames_in_flight)
        : m_device(device)
        , m_max_frames_in_flight(max_frames_in_flight)
      {}

    public:
      virtual ~Synchronizer() = default;

      static fun make(rhi::Device &device, u32 max_frames_in_flight = 2) {
        return make_uptr(MochiRHI_MakeSynchronizer(device, max_frames_in_flight));
      }

    protected:
      rhi::Device &m_device;
      u32 m_max_frames_in_flight;

    public:
      virtual fun beginFrame() -> void = 0;
      virtual fun endFrame() -> void = 0;

      virtual fun currentFrameIndex() const -> u32 = 0;
      virtual fun maxFramesInFlight() const -> u32 = 0;

      virtual fun activeInFlightFence() -> void* = 0;
      virtual fun activeImageAvailableSemaphore() -> void* = 0;
  };
  
}
