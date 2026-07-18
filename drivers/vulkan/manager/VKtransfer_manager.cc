/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "drivers/vulkan/manager/VKtransfer_manager.hh"
#include "drivers/vulkan/VKbuffer.hh"
#include "drivers/vulkan/VKimage.hh"
#include <cstring>
#include <vulkan/vulkan.hpp>



namespace mochi::rhi::vulkan::mng
{
  extern "C" fun MochiRHI_MakeTransferManager(rhi::mng::DeviceManager &dmng) -> rhi::mng::TransferManager* {
    return new VK_TransferManager(dmng);
  }

  VK_TransferManager::VK_TransferManager(rhi::mng::DeviceManager &dmng)
    : rhi::mng::TransferManager(dmng)
  {}

  fun VK_TransferManager::copyMemoryToImage(rhi::mng::TransferTime time, void* src, rhi::Image2* dst) -> void {
    auto& vk_dmng = static_cast<VK_DeviceManager&>(m_dmng);
    auto* vk_dst = static_cast<VK_Image2*>(dst);

    vk::MemoryToImageCopy copyRegion(
      src, 0, 0,
      vk::ImageSubresourceLayers(vk::ImageAspectFlagBits::eColor, 0, 0, 1),
      vk::Offset3D(0, 0, 0),
      vk::Extent3D(vk_dst->ext().x(), vk_dst->ext().y(), 1)
    );

    vk::CopyMemoryToImageInfo copyInfo(
      {}, vk_dst->get(), vk::ImageLayout::eGeneral, 1, &copyRegion
    );

    vk::HostImageLayoutTransitionInfo transitionInfo(
      vk_dst->get(), vk::ImageLayout::eUndefined, vk::ImageLayout::eGeneral,
      vk::ImageSubresourceRange(vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1)
    );
    vk_dmng.get().transitionImageLayoutEXT(transitionInfo);

    vk_dmng.get().copyMemoryToImageEXT(copyInfo);

    vk::HostImageLayoutTransitionInfo transitionInfo2(
      vk_dst->get(), vk::ImageLayout::eGeneral, vk::ImageLayout::eShaderReadOnlyOptimal,
      vk::ImageSubresourceRange(vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1)
    );
    vk_dmng.get().transitionImageLayoutEXT(transitionInfo2);
  }

  fun VK_TransferManager::copyMemoryToBuffer(rhi::mng::TransferTime time, void* src, rhi::Buffer* dst) -> void {
    auto* vk_dst = static_cast<VK_Buffer*>(dst);
    
    // Direct mapping if mapped
    if (vk_dst->mapped()) {
      std::memcpy(vk_dst->mapped(), src, vk_dst->size());
    } else {
      // Staging buffer fallback logic goes here
    }
  }
}
