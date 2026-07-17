/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/render_target.hh"
#include "mochi/types.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/rhi/manager/material_manager.hh"
#include <vulkan/vulkan_raii.hpp>



namespace mochi::rhi::vulkan
{
  
  struct VK_MaterialManager final: rhi::MaterialManager {
    public:
      VK_MaterialManager(rhi::DeviceManager &device, ShaderManager &smng);

    public:
      std::unordered_map<MaterialProps, sptr<MaterialDesc>> m_materials;

    public:
      fun getMaterialDesc(rhi::RenderTarget &target, MaterialProps props) -> sptr<MaterialDesc>;
  };
  
}
