/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/command.hh"
#include "mochi/types.hh"
#include "mochi/asset/scene.hh"
#include "mochi/rhi/manager/allocator.hh"
#include "mochi/rhi/manager/device.hh"
#include "mochi/rhi/manager/synchronizer.hh"
#include <cassert>



namespace mochi::sys
{

  struct RenderingSystem: noncopy {
    public:
      static constexpr u32 MAX_FRAMES_IN_FLIGHT = 2;

    public:
      explicit RenderingSystem(rhi::Device &dmng, rhi::Allocator &amng, rhi::CommandManager &cmng, rhi::Synchronizer &sync);
      ~RenderingSystem();

    private:
      rhi::Device         &m_device;
      rhi::Allocator      &m_alloc;
      rhi::CommandManager &m_cmng;
      rhi::Synchronizer   &m_sync;

      std::vector<sptr<rhi::Command>> m_cmd_buffers;

      sptr<rhi::Buffer> m_light_ubo;
      sptr<rhi::Buffer> m_camera_ubo;

    public:
      fun current_frame() const { return m_sync.currentFrameIndex(); }
      fun get_image_available_sem() -> void* { return m_sync.activeImageAvailableSemaphore(); }

    public:
      fun draw(rhi::Command &cmd, rhi::RenderTarget &target, asset::Scene *scene, sys::DisplaySystem &disp) -> void;
      
      fun begin_pass(rhi::Command &cmd, const rhi::RenderTarget &target, const std::array<f32,4> &clear_color) -> void;
      fun end_pass(rhi::Command &cmd, const rhi::RenderTarget &target) -> void;

      fun begin_frame() -> rhi::Command&;
      fun end_frame(rhi::Command &cmd, void* wait_sem, void* signal_sem) -> void;

    public:
      fun light_ubo() { return m_light_ubo; }
      fun camera_ubo() { return m_camera_ubo; }

    private:
      fun prepare_light_ubo(u64 required_count) -> void;
      fun prepare_camera_ubo(u64 required_count) -> void;
  };

}
