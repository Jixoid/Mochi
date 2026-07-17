/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/types.hh"
#include "mochi/rhi/shader.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "drivers/vulkan/manager/VKdevice_manager.hh"
#include "drivers/vulkan/VKshader.hh"


namespace mochi::rhi::vulkan
{

  extern "C" fun MochiRHI_MakeShader(rhi::DeviceManager &device, ShaderStage stage, std::span<u32> span, std::string_view entry) -> Shader* {
    return new VK_Shader(device, stage, span, entry);
  }
  

  VK_Shader::VK_Shader(rhi::DeviceManager &device, ShaderStage stage, std::span<u32> span, std::string_view entry) {
    m_stage = stage;
    m_entry = entry;

    vk::ShaderModuleCreateInfo info({}, span.size_bytes(), span.data());
  
    vk_module = vk::raii::ShaderModule(static_cast<VK_DeviceManager&>(device).get(), info);
  }

}
