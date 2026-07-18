/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "drivers/vulkan/manager/VKcommand_manager.hh"
#include "drivers/vulkan/manager/VKdevice_manager.hh"
#include "drivers/vulkan/VKcommand.hh"
#include <vulkan/vulkan_raii.hpp>

namespace mochi::rhi::vulkan::mng
{
  extern "C" fun MochiRHI_MakeCommandManager(rhi::mng::DeviceManager &dmng) -> rhi::mng::CommandManager* {
    return new VK_CommandManager(dmng);
  }

  VK_CommandManager::VK_CommandManager(rhi::mng::DeviceManager &dmng)
    : rhi::mng::CommandManager(dmng)
  {}

  fun VK_CommandManager::allocateGraphicsCommand() -> sptr<Command> {
    auto& vk_dmng = static_cast<VK_DeviceManager&>(m_dmng);
    // For now we use the main pool/buffer logic. In reality this should use proper command pools.
    auto cmd = vk_dmng.getMainBuffer(1);
    // VK_Command needs to take ownership of this command buffer
    // (You will need to update VK_Command to support this construction)
    // For now I'm constructing it with an empty constructor and we will fix VK_Command next.
    return make_sptr(new VK_Command(std::move(cmd.front())));
  }

  fun VK_CommandManager::allocateComputeCommand() -> sptr<Command> {
    return allocateGraphicsCommand(); // Defaulting to main for now
  }

  fun VK_CommandManager::allocateTransferCommand() -> sptr<Command> {
    // Actually we removed transfer pool, so we just use the main pool or throw since transfer shouldn't need a command buffer for direct host copy!
    // But if we need it for staging fallback, we can use main pool.
    return allocateGraphicsCommand();
  }

  fun VK_CommandManager::submit(Command* cmd) -> void {
    auto* vk_cmd = static_cast<VK_Command*>(cmd);
    vk::SubmitInfo submit_info({}, {}, *vk_cmd->get(), {});
    
    auto& vk_dmng = static_cast<VK_DeviceManager&>(m_dmng);
    u32 family = vk_dmng.main_q().family();
    
    auto& queue = vk_dmng.active_queue(family);
    queue.submit(submit_info, nullptr);
  }

  fun VK_CommandManager::submitWithSync(Command* cmd, void* waitSem, void* sigSem, void* fence) -> void {
    auto* vk_cmd = static_cast<VK_Command*>(cmd);
    
    vk::Semaphore wait_semaphore = static_cast<VkSemaphore>(waitSem);
    vk::Semaphore signal_semaphore = static_cast<VkSemaphore>(sigSem);
    vk::Fence submit_fence = static_cast<VkFence>(fence);
    vk::PipelineStageFlags wait_stages[] = {vk::PipelineStageFlagBits::eColorAttachmentOutput};

    vk::SubmitInfo submit_info(
      waitSem ? 1 : 0, waitSem ? &wait_semaphore : nullptr, wait_stages,
      1, &vk_cmd->get().operator*(),
      sigSem ? 1 : 0, sigSem ? &signal_semaphore : nullptr
    );

    auto& vk_dmng = static_cast<VK_DeviceManager&>(m_dmng);
    u32 family = vk_dmng.main_q().family();
    
    auto& queue = vk_dmng.active_queue(family);
    queue.submit(submit_info, submit_fence);
  }
}
