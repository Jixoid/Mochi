/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/except.hh"
#include "mochi/module/renderer.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/rhi/manager/command_manager.hh"
#include "mochi/rhi/manager/sync_manager.hh"

#define ef else if


namespace mochi::module
{

  renderer::renderer(rhi::DeviceManager &device)
    : m_device(device)
  {
    m_cmd_mgr = rhi::CommandManager::make(m_device);
    m_sync_mgr = rhi::SyncManager::make(m_device, MAX_FRAMES_IN_FLIGHT);

    for (u32 i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
      m_cmd_buffers.push_back(m_cmd_mgr->allocateGraphicsCommand());
    }
  }


  fun renderer::begin_pass(rhi::Command &cmd, const rhi::render_target &target, const std::array<float, 4> &clear_color) -> void
  {
    // Normally this logic belongs in rhi::Command::beginRendering!
    // Since we are refactoring out vulkan completely from this module,
    // this module should not know about vk::ImageMemoryBarrier or vk::RenderingAttachmentInfo.
    // However, the rhi::Command interface currently lacks these features.
    // For now, this is a placeholder indicating where Command interface needs expansion.
  }

  fun renderer::end_pass(rhi::Command &cmd, const rhi::render_target &target) -> void
  {
    // Same here, should call cmd.endRendering() and cmd.transition(...)
  }

  fun renderer::begin_frame() -> rhi::Command&
  {
    m_sync_mgr->beginFrame();
    
    auto& cmd = *m_cmd_buffers[m_sync_mgr->currentFrameIndex()];
    // Should call cmd.begin()
    
    return cmd;
  }

  fun renderer::end_frame(rhi::Command &cmd, void* wait_sem, void* signal_sem) -> void
  {
    // Should call cmd.end()
    
    m_cmd_mgr->submitWithSync(&cmd, wait_sem, signal_sem, m_sync_mgr->activeInFlightFence());
    
    m_sync_mgr->endFrame();
  }

}
