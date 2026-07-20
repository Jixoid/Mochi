/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/rhi/utility/shader_compile_utility.hh"
#include "mochi/rhi/shader.hh"
#include "mochi/rhi/manager/device.hh"
#include <shaderc/shaderc.hpp>
#include <vulkan/vulkan_raii.hpp>
#include <string>
#include <string_view>
#include <span>



namespace mochi::rhi::vulkan
{

  struct VK_ShaderCompileUtility final: rhi::ShaderCompileUtility {
    public:
      explicit VK_ShaderCompileUtility(rhi::Device &device): rhi::ShaderCompileUtility(device) {}

    private:
      shaderc::Compiler m_compiler;

    public:
      fun compileGLSL(
        u64 sign, ShaderStage stage, std::span<char> code = {},
        std::span<std::string> macros = {},
        std::string_view entry = "main"
      ) -> std::vector<u8> override;
  };

}
