/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "drivers/vulkan/VKdriver.hh"
#include "drivers/vulkan/manager/VKshader_manager.hh"
#include "drivers/vulkan/VKshader.hh"
#include "mochi/basis.hh"
#include "mochi/debug/debug.hh"
#include "mochi/rhi/manager/shader_manager.hh"
#include "mochi/types.hh"
#include "mochi/rhi/shader.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/vfs/vfs.hh"
#include <expected>
#include <format>
#include <string>
#include <string_view>
#include <span>



namespace mochi::rhi::vulkan
{

  extern "C" fun MochiRHI_MakeShaderManager(rhi::DeviceManager &device) -> ShaderManager* {
    return new VK_ShaderManager(device);
  }
  

  fun VK_ShaderManager::compileGLSL(
    u64 sign, ShaderStage stage, std::span<char> code,
    std::span<std::string> macros, std::string_view entry
  ) -> sptr<Shader>
  {
    auto path = std::format(".cache/mochi/vulkan/shaders/{:x}.spv", sign);

    if (sign != 0) {  
      if (vfs::exists(path)) {
        ME_LOG_VERB("shader reading")

        auto file = vfs::open_map(path);
        std::span<u32> code((u32*)file->data(), file->size()/4);
        
        return make_sptr(new VK_Shader(m_dmng, stage, code, entry));
      }
    }

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

    
    if (sign != 0) {
      ME_LOG_VERB("shader writing")

      vfs::open_rw(path)->write((char*)module.begin(), (module.end()-module.begin())*4).flush();
    }
    
    return make_sptr(new VK_Shader(m_dmng, stage, std::span<u32>((u32*)module.begin(), module.end()), entry));
  }

}
