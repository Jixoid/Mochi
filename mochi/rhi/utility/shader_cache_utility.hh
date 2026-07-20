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



namespace mochi::rhi
{

  // External
  extern "C" fun MochiRHI_MakeShaderCacheUtility(rhi::Device &device) -> ShaderCacheUtility*;
  

  // Interface
  struct ShaderCacheUtility: noncopy {
    protected:
      ShaderCacheUtility(rhi::Device &device): m_device(device) {}

    public:
      virtual ~ShaderCacheUtility() = default;

      static fun make(rhi::Device &device) {
        return make_uptr(MochiRHI_MakeShaderCacheUtility(device));
      }

    protected:
      rhi::Device &m_device;

    public:
      virtual fun loadCache(u64 sign) -> std::vector<u8> = 0;
      virtual fun saveCache(u64 sign, std::span<const u8> data) -> void = 0;
  };

}
