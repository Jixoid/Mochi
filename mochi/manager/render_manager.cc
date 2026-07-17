/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/manager/render_manager.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/rhi/manager/command_manager.hh"
#include "mochi/rhi/manager/sync_manager.hh"
#include "mochi/rhi/render_target.hh"

#define ef else if



namespace mochi::manager
{

  RenderManager::RenderManager(rhi::DeviceManager &dmng): m_dmng(dmng) {
    m_cmd_mgr = rhi::CommandManager::make(m_dmng);
    m_sync_mgr = rhi::SyncManager::make(m_dmng, MAX_FRAMES_IN_FLIGHT);

    for (u32 i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
      m_cmd_buffers.push_back(m_cmd_mgr->allocateGraphicsCommand());
    }
  }


  fun RenderManager::begin_frame() -> rhi::Command& {
    m_sync_mgr->beginFrame();

    auto &cmd = *m_cmd_buffers[current_frame()];
    
    cmd.begin();
    
    if (m_dmng.descriptor_heap()) {
      cmd.bindDescriptorHeap(m_dmng.descriptor_heap(), m_dmng.sampler_heap(), m_dmng.descriptor_heap()->size(), m_dmng.sampler_heap()->size());
    }

    return cmd;
  }

  fun RenderManager::end_frame(rhi::Command &cmd, void *wait_sem, void *signal_sem) -> void {
    cmd.end();
    
    m_cmd_mgr->submitWithSync(&cmd, wait_sem, signal_sem, m_sync_mgr->activeInFlightFence());

    m_sync_mgr->endFrame();
  }


  fun RenderManager::begin_pass(rhi::Command &cmd, const rhi::RenderTarget &target, const std::array<f32,4> &clear_color) -> void {
    cmd.beginRendering(target, clear_color);
  }

  fun RenderManager::end_pass(rhi::Command &cmd, const rhi::RenderTarget &target) -> void {
    cmd.endRendering(target);
  }

}
