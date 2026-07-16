/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/rhi/shader.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include <shaderc/shaderc.hpp>
#include <vulkan/vulkan_raii.hpp>
#include <string>
#include <string_view>
#include <span>



namespace mochi::rhi::vulkan
{

  struct VK_Shader final: public rhi::Shader {
    public:
      explicit VK_Shader(rhi::DeviceManager &device, ShaderStage stage, std::span<u32> span, std::string_view entry);
      ~VK_Shader() = default;

    private:
      std::string m_entry;
      vk::raii::ShaderModule vk_module{nil};
      ShaderStage m_stage;

    public:
      fun entry() const -> const std::string& override { return m_entry; }
      fun stage() const -> ShaderStage override { return m_stage; }
      
    public:
      fun& get() { return vk_module; }
  };

}
