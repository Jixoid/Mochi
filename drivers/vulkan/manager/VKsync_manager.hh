/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/manager/sync_manager.hh"
#include <vulkan/vulkan_raii.hpp>


namespace mochi::rhi::vulkan::mng
{
  struct VK_SyncManager final : public rhi::mng::SyncManager {
    public:
      explicit VK_SyncManager(rhi::mng::DeviceManager &dmng, u32 max_frames_in_flight);
      ~VK_SyncManager() override = default;

    private:
      std::vector<vk::raii::Semaphore> m_image_available_sems;
      std::vector<vk::raii::Fence>     m_in_flight_fences;

      u32 m_current_frame{};

    public:
      fun beginFrame() -> void override;
      fun endFrame() -> void override;

      fun currentFrameIndex() const -> u32 override { return m_current_frame; }
      fun maxFramesInFlight() const -> u32 override { return m_max_frames_in_flight; }

      fun activeInFlightFence() -> void* override;
      fun activeImageAvailableSemaphore() -> void* override;
  };
}
