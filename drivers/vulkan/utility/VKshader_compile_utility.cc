/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "drivers/vulkan/utility/VKshader_compile_utility.hh"
#include "mochi/basis.hh"
#include "mochi/debug/debug.hh"
#include "mochi/rhi/utility/shader_compile_utility.hh"
#include "mochi/types.hh"
#include "mochi/rhi/shader.hh"
#include "mochi/rhi/manager/device.hh"
#include <cstring>
#include <expected>
#include <format>
#include <string>
#include <string_view>
#include <span>
#include <unordered_map>
#include <vector>



namespace mochi::rhi::vulkan
{

  extern "C" fun MochiRHI_MakeShaderCompileUtility(rhi::Device &device) -> rhi::ShaderCompileUtility* {
    return new VK_ShaderCompileUtility(device);
  }
  

  fun VK_ShaderCompileUtility::compileGLSL(
    u64 sign, ShaderStage stage, std::span<char> code,
    std::span<std::string> macros, std::string_view entry
  ) -> std::vector<u8>
  {
    ME_LOG_VERB("shader compiling")

    static std::unordered_map<rhi::ShaderStage, shaderc_shader_kind> ToKind = {
      {rhi::ShaderStage::Vertex,  shaderc_glsl_vertex_shader},
      {rhi::ShaderStage::Pixel,   shaderc_glsl_fragment_shader},
      {rhi::ShaderStage::Compute, shaderc_glsl_compute_shader},
    };

    shaderc::CompileOptions options;

    options.SetTargetEnvironment(shaderc_target_env_vulkan, shaderc_env_version_vulkan_1_4);
    options.SetOptimizationLevel(shaderc_optimization_level_performance);

    for (auto &M: macros)
      options.AddMacroDefinition(M);

    auto module = m_compiler.CompileGlslToSpv(code.data(), code.size(), ToKind[stage], "main.glsl", options);
    
    if (module.GetCompilationStatus() != shaderc_compilation_status_success)
      throw std::unexpected("Shader compilation error: " + module.GetErrorMessage());

    ME_LOG_VERB("shader compiled")

    auto beg = (u8*)module.begin();
    auto size = usize((u8*)module.end() - (u8*)module.begin());

    std::vector<u8> vec(size);

    std::memcpy(vec.data(), beg, size);

    return std::move(vec);
  }

}
