/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "drivers/vulkan/manager/VKsync_manager.hh"
#include "drivers/vulkan/manager/VKdevice_manager.hh"

namespace mochi::rhi::vulkan
{
  extern "C" fun MochiRHI_MakeSyncManager(rhi::DeviceManager &dmng, u32 max_frames_in_flight) -> SyncManager* {
    return new VK_SyncManager(dmng, max_frames_in_flight);
  }

  VK_SyncManager::VK_SyncManager(rhi::DeviceManager &dmng, u32 max_frames_in_flight)
    : rhi::SyncManager(dmng, max_frames_in_flight)
  {
    auto& vk_dmng = static_cast<VK_DeviceManager&>(m_dmng);
    
    for (u32 i = 0; i < m_max_frames_in_flight; i++) {
      m_image_available_sems.push_back(vk::raii::Semaphore(vk_dmng.get(), vk::SemaphoreCreateInfo()));
      m_in_flight_fences.push_back(vk::raii::Fence(vk_dmng.get(), vk::FenceCreateInfo(vk::FenceCreateFlagBits::eSignaled)));
    }
  }

  fun VK_SyncManager::beginFrame() -> void {
    auto& vk_dmng = static_cast<VK_DeviceManager&>(m_dmng);
    vk::Result res;
    do {
      res = vk_dmng.get().waitForFences({*m_in_flight_fences[m_current_frame]}, VK_TRUE, UINT64_MAX);
    } while (res == vk::Result::eTimeout);
    
    vk_dmng.get().resetFences({*m_in_flight_fences[m_current_frame]});
  }

  fun VK_SyncManager::endFrame() -> void {
    m_current_frame = (m_current_frame + 1) % m_max_frames_in_flight;
  }

  fun VK_SyncManager::activeInFlightFence() -> void* {
    if (m_in_flight_fences.empty()) return nullptr;
    return (void*)(VkFence)(*m_in_flight_fences[m_current_frame]);
  }

  fun VK_SyncManager::activeImageAvailableSemaphore() -> void* {
    if (m_image_available_sems.empty()) return nullptr;
    return (void*)(VkSemaphore)(*m_image_available_sems[m_current_frame]);
  }
  
}
