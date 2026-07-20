/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/manager/uploader.hh"
#include "drivers/vulkan/manager/VKdevice.hh"
#include <vulkan/vulkan_raii.hpp>


namespace mochi::rhi::vulkan
{
  struct VK_Uploader final : public rhi::Uploader {
    public:
      explicit VK_Uploader(rhi::Device &dmng);
      ~VK_Uploader() override = default;

    public:
      fun copyMemoryToImage(rhi::TransferTime time, void* src, rhi::Image2* dst) -> void override;
      fun copyMemoryToBuffer(rhi::TransferTime time, void* src, rhi::Buffer* dst) -> void override;
  };
}
