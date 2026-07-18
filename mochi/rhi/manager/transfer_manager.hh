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


namespace mochi::rhi::mng
{
  // Enums
  enum struct TransferTime {
    Now,
    Lazy,
  };


  // External
  extern "C" fun MochiRHI_MakeTransferManager(rhi::mng::DeviceManager &dmng) -> TransferManager*;


  // Interface
  struct TransferManager: noncopy {
    protected:
      TransferManager(rhi::mng::DeviceManager &dmng): m_dmng(dmng) {}

    public:
      virtual ~TransferManager() = default;

      static fun make(rhi::mng::DeviceManager &device) {
        return make_uptr(MochiRHI_MakeTransferManager(device));
      }

    protected:
      rhi::mng::DeviceManager &m_dmng;

    public:
      virtual fun copyMemoryToImage(TransferTime time, void* src, rhi::Image2* dst) -> void = 0;
      virtual fun copyMemoryToBuffer(TransferTime time, void* src, rhi::Buffer* dst) -> void = 0;
  };

}
