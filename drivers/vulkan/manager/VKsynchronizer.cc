/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "drivers/vulkan/manager/VKsynchronizer.hh"
#include "drivers/vulkan/manager/VKdevice.hh"



namespace mochi::rhi::vulkan
{

  extern "C" fun MochiRHI_MakeSynchronizer(rhi::Device &dmng, u32 max_frames_in_flight) -> Synchronizer* {
    return new VK_Synchronizer(dmng, max_frames_in_flight);
  }

  
  VK_Synchronizer::VK_Synchronizer(rhi::Device &dmng, u32 max_frames_in_flight)
    : rhi::Synchronizer(dmng, max_frames_in_flight)
  {
    auto& vk_dmng = static_cast<VK_Device&>(m_device);
    
    for (u32 i = 0; i < m_max_frames_in_flight; i++) {
      m_image_available_sems.push_back(vk::raii::Semaphore(vk_dmng.get(), vk::SemaphoreCreateInfo()));
      m_in_flight_fences.push_back(vk::raii::Fence(vk_dmng.get(), vk::FenceCreateInfo(vk::FenceCreateFlagBits::eSignaled)));
    }
  }

  fun VK_Synchronizer::beginFrame() -> void {
    auto& vk_dmng = static_cast<VK_Device&>(m_device);
    vk::Result res;
    do {
      res = vk_dmng.get().waitForFences({*m_in_flight_fences[m_current_frame]}, VK_TRUE, UINT64_MAX);
    } while (res == vk::Result::eTimeout);
    
    vk_dmng.get().resetFences({*m_in_flight_fences[m_current_frame]});
  }

  fun VK_Synchronizer::endFrame() -> void {
    m_current_frame = (m_current_frame + 1) % m_max_frames_in_flight;
  }

  fun VK_Synchronizer::activeInFlightFence() -> void* {
    if (m_in_flight_fences.empty()) return nullptr;
    return (void*)(VkFence)(*m_in_flight_fences[m_current_frame]);
  }

  fun VK_Synchronizer::activeImageAvailableSemaphore() -> void* {
    if (m_image_available_sems.empty()) return nullptr;
    return (void*)(VkSemaphore)(*m_image_available_sems[m_current_frame]);
  }
  
}
