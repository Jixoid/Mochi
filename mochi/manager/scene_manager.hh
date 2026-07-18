/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/rhi/manager/alloc_manager.hh"
#include "mochi/types.hh"
#include <cassert>



namespace mochi::mng
{

  struct SceneManager: noncopy {
    public:
      explicit SceneManager(rhi::mng::DeviceManager &dmng, rhi::mng::AllocManager &amng);
      ~SceneManager();


    private:
      rhi::mng::DeviceManager &m_dmng;
      rhi::mng::AllocManager &m_amng;

    
    private:
      sptr<rhi::Buffer> m_camera_ubo;
      sptr<rhi::Buffer> m_light_ubo;

    public:
      fun camera_ubo(u64 required_count = 0) -> sptr<rhi::Buffer>;
      fun light_ubo(u64 required_count = 0) -> sptr<rhi::Buffer>;
  };

}
