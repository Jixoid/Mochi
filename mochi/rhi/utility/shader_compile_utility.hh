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
  enum struct ShaderCompileOptimization {
    Zero,
    Size,
    Performance,
  };


  // External
  extern "C" fun MochiRHI_MakeShaderCompileUtility(rhi::Device &device) -> ShaderCompileUtility*;
  

  // Interface
  struct ShaderCompileUtility: noncopy {
    protected:
      ShaderCompileUtility(rhi::Device &device): m_device(device) {}

    public:
      virtual ~ShaderCompileUtility() = default;

      static fun make(rhi::Device &device) {
        return make_uptr(MochiRHI_MakeShaderCompileUtility(device));
      }

    protected:
      rhi::Device &m_device;
      ShaderCompileOptimization m_opt;

    public:
      fun& optimization() { return m_opt; }
    
    public:
      virtual fun compileGLSL(
        u64 sign, ShaderStage stage, std::span<char> code = {},
        std::span<std::string> macros = {},
        std::string_view entry = "main"
      ) -> std::vector<u8> = 0;
  };

}
