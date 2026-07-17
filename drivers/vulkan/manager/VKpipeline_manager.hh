/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/rhi/manager/pipeline_manager.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include <span>
#include <vector>



namespace mochi::rhi::vulkan
{

  struct VK_PipelineManager final: rhi::PipelineManager {
    public:
      explicit VK_PipelineManager(rhi::DeviceManager &device): rhi::PipelineManager(device) {}

    public:
      fun loadCache(u64 sign) -> std::vector<u8> override;
      fun saveCache(u64 sign, std::span<const u8> data) -> void override;
  };

}
