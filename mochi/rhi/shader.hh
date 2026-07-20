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
#include <string_view>
#include <span>


namespace mochi::rhi
{
  // Enums
  enum struct ShaderStage: u32 {
    Vertex       = 0x00000001,
    Pixel        = 0x00000010,
    Compute      = 0x00000020,
    Task         = 0x00000040,
    Mesh         = 0x00000080,
    RayGen       = 0x00000100,
    AnyHit       = 0x00000200,
    ClosestHit   = 0x00000400,
    Miss         = 0x00000800,
    Intersection = 0x00001000,
    Callable     = 0x00002000,
  };
  using ShaderStageFlags = flags<ShaderStage>;


  // External
  extern "C" fun MochiRHI_MakeShader(rhi::Device &device, ShaderStage stage, std::span<u8> span, std::string_view entry) -> Shader*;
  

  // Interface
  struct Shader: noncopy {
    protected:
      Shader() = default;
    
    public:
      virtual ~Shader() = default;
      
      static fun make(rhi::Device &device, ShaderStage stage, std::span<u8> span, std::string_view entry = "main"){
        return make_sptr(MochiRHI_MakeShader(device, stage, span, entry));
      }

    public:
      virtual fun entry() const -> const std::string& = 0;
      virtual fun stage() const -> ShaderStage = 0;
  };

}

FlagEnable(mochi::rhi::ShaderStage)
