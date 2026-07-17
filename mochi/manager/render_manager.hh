/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/types.hh"
#include "mochi/rhi/render_target.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/rhi/manager/command_manager.hh"
#include "mochi/rhi/manager/sync_manager.hh"
#include "mochi/rhi/command.hh"



namespace mochi::manager
{

  struct RenderManager: noncopy {
    public:
      static constexpr u32 MAX_FRAMES_IN_FLIGHT = 2;

    public:
      explicit RenderManager(rhi::DeviceManager &dmng);


    private:
      rhi::DeviceManager &m_dmng;

      sptr<rhi::CommandManager> m_cmd_mgr;
      sptr<rhi::SyncManager>    m_sync_mgr;

      std::vector<sptr<rhi::Command>> m_cmd_buffers;

    public:
      fun current_frame() const { return m_sync_mgr->currentFrameIndex(); }
      fun get_image_available_sem() -> void* { return m_sync_mgr->activeImageAvailableSemaphore(); }

    public:
      fun begin_pass(rhi::Command &cmd, const rhi::RenderTarget &target, const std::array<f32,4> &clear_color) -> void;
      fun end_pass(rhi::Command &cmd, const rhi::RenderTarget &target) -> void;

      fun begin_frame() -> rhi::Command&;
      fun end_frame(rhi::Command &cmd, void* wait_sem, void* signal_sem) -> void;
  };
  
}
