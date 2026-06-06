/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/rhi.hh"
#include "mochi/types.hh"
#include <string_view>
#include <span>
#include <vulkan/vulkan_raii.hpp>



namespace mochi::rhi
{

  struct shader
  {
    private:
      explicit shader(rhi::device &device, ShaderStage stage, std::span<u32> span, std::string_view entry);
    
    public:
      static inline fun make(rhi::device &device, ShaderStage stage, std::span<u32> span, std::string_view entry) {
        return make_sptr(new shader(device, stage, span, entry));
      }


    protected:
      std::string m_entry;
      vk::raii::ShaderModule vk_module;
      ShaderStage m_stage;

    public:
      inline fun& entry() { return m_entry; }
      inline fun  stage() { return m_stage; }
      inline fun& module() { return vk_module; }
  };

}
