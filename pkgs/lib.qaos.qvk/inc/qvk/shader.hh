/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "qvk/types.hh"
#include <string_view>
#include <vulkan/vulkan_raii.hpp>



namespace qvk
{

  struct shader
  {
    public:
      explicit shader(qvk::core &core, std::string_view fpath, std::string_view entry);
      

    private:
      vk::raii::ShaderModule vk_module;
      std::string m_entry;

    public:
      inline fun& module() { return vk_module; }
      inline fun& entry() { return m_entry; }
  };

}
