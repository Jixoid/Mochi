/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/rhi/image.hh"
#include "mochi/rhi/sampler.hh"
#include "mochi/types.hh"



namespace mochi::rhi::mng
{
  // External
  extern "C" fun MochiRHI_MakeResourceManager(rhi::mng::DeviceManager &dmng) -> ResourceManager*;



  // Interface
  struct ResourceManager: noncopy {
    protected:
      ResourceManager(rhi::mng::DeviceManager &dmng): m_dmng(dmng) {}

    public:
      virtual ~ResourceManager() = default;

      static fun make(rhi::mng::DeviceManager &device) {
        return make_sptr(MochiRHI_MakeResourceManager(device));
      }

    protected:
      rhi::mng::DeviceManager &m_dmng;

    public:
      // Kaynakları silinmek üzere kuyruğa alır
      virtual fun deferDestroy(sptr<Buffer> buffer) -> void = 0;
      virtual fun deferDestroy(sptr<Image2> image) -> void = 0;
      virtual fun deferDestroy(sptr<Sampler2> sampler) -> void = 0;

      // Her kare sonunda çağrılarak ömrü dolan (GPU işi biten) kaynakları gerçekten siler
      virtual fun tick(u32 currentFrameIndex) -> void = 0;
  };

}
