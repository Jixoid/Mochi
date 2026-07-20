/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "drivers/vulkan/manager/VKallocator.hh"
#include "mochi/basis.hh"
#include "mochi/types.hh"
#include "vk_mem_alloc.h"
#include "mochi/rhi/manager/device.hh"
#include "drivers/vulkan/manager/VKdevice.hh"
#include "drivers/vulkan/VKbuffer.hh"
#include "drivers/vulkan/VKimage.hh"
#include <vulkan/vulkan_raii.hpp>


namespace mochi::rhi::vulkan
{

  VK_Image2::VK_Image2(rhi::Device &device, VmaAllocator vma_allocator, VkImage image, VmaAllocation allocation, extent<2,u32> ext)
    : m_device(device)
    , m_vma_allocator(vma_allocator)
    , m_allocation(allocation)
    , m_image(image)
    , m_ext(ext)
  {
  }

  VK_Image2::~VK_Image2() {
    if (m_vma_allocator)
      vmaDestroyImage(m_vma_allocator, m_image, m_allocation);
  }



  fun VK_Image2::makeView() const -> sptr<ImageView2> {
    vk::ImageViewCreateInfo view_info(
      {}, m_image, vk::ImageViewType::e2D, vk::Format::eR8G8B8A8Unorm,
      {}, {vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1}
    );
    auto e_view = vk::raii::ImageView(static_cast<vulkan::VK_Device&>(m_device).get(), view_info);

    return make_sptr(new VK_ImageView2(std::move(e_view)));
  }

}
