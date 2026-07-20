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



namespace mochi::rhi
{
  // Enums
  enum struct TransferTime {
    Now,
    Lazy,
  };


  // External
  extern "C" fun MochiRHI_MakeUploader(rhi::Device &device) -> Uploader*;


  // Interface
  struct Uploader: noncopy {
    protected:
      Uploader(rhi::Device &device): m_device(device) {}

    public:
      virtual ~Uploader() = default;

      static fun make(rhi::Device &device) {
        return make_uptr(MochiRHI_MakeUploader(device));
      }

    protected:
      rhi::Device &m_device;

    public:
      virtual fun copyMemoryToImage(TransferTime time, void* src, rhi::Image2* dst) -> void = 0;
      virtual fun copyMemoryToBuffer(TransferTime time, void* src, rhi::Buffer* dst) -> void = 0;
  };

}
