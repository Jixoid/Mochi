/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/manager/transfer_manager.hh"
#include "drivers/vulkan/manager/VKdevice_manager.hh"
#include <vulkan/vulkan_raii.hpp>

namespace mochi::rhi::vulkan
{
  struct VK_TransferManager final : public rhi::TransferManager {
    public:
      explicit VK_TransferManager(rhi::DeviceManager &dmng);
      ~VK_TransferManager() override = default;

    public:
      fun copyMemoryToImage(TransferTime time, void* src, rhi::Image2* dst) -> void override;
      fun copyMemoryToBuffer(TransferTime time, void* src, rhi::Buffer* dst) -> void override;
  };
}
