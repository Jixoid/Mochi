/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "qvk/module/device.hh"
#include "qvk/core.hh"
#include "qvk/types.hh"
#include "qvk/rhi/shader.hh"
#include <string>
#include <string_view>
#include <vulkan/vulkan_raii.hpp>



namespace qvk
{

  shader::shader(core &core, std::string_view fpath, std::string_view entry)
    : vk_module(nil)
    , m_entry(entry)
  {
    mappedFile mfile((std::string)fpath);
    

    // Load Shader
    vk::ShaderModuleCreateInfo info({}, mfile.view().size(), (u32*)mfile.view().data());
      
    vk_module = vk::raii::ShaderModule(core.sub<module::device>().vdevice(), info);
  }

}
