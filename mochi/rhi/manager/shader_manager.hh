/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/shader.hh"
#include "mochi/types.hh"


namespace mochi::rhi
{
  // Enums
  enum struct ShaderCompOptimization {
    Zero,
    Size,
    Performance,
  };

  
  // External
  extern "C" fun MochiRHI_MakeShaderManager(rhi::DeviceManager &dmng) -> ShaderManager*;


  // Interface
  struct ShaderManager: noncopy {
    protected:
      ShaderManager(rhi::DeviceManager &dmng): m_dmng(dmng) {}

    public:
      virtual ~ShaderManager() = default;

      static fun make(rhi::DeviceManager &device) {
        return make_uptr(MochiRHI_MakeShaderManager(device));
      }

    protected:
      rhi::DeviceManager &m_dmng;
      ShaderCompOptimization m_opt;

    public:
      fun& optimization() { return m_opt; }
    
    public:
      virtual fun compileGLSL(
        u64 sign, ShaderStage stage, std::span<char> code = {},
        std::span<std::string> macros = {},
        std::string_view entry = "main"
      ) -> sptr<Shader> = 0;
  };

}
