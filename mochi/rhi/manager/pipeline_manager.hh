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
#include <span>
#include <vector>


namespace mochi::rhi
{
  // External
  extern "C" fun MochiRHI_MakePipelineManager(rhi::DeviceManager &dmng) -> PipelineManager*;


  // Interface
  struct PipelineManager: noncopy {
    protected:
      PipelineManager(rhi::DeviceManager &dmng): m_dmng(dmng) {}

    public:
      virtual ~PipelineManager() = default;

      static fun make(rhi::DeviceManager &device) {
        return make_uptr(MochiRHI_MakePipelineManager(device));
      }

    protected:
      rhi::DeviceManager &m_dmng;

    public:
      virtual fun loadCache(u64 sign) -> std::vector<u8> = 0;
      virtual fun saveCache(u64 sign, std::span<const u8> data) -> void = 0;
  };

}
