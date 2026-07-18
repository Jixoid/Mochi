/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/rhi/image.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include <vulkan/vulkan_raii.hpp>
#include "vk_mem_alloc.h"



namespace mochi::rhi::vulkan
{

  struct VK_Image2 final: public rhi::Image2 {
    public:
      explicit VK_Image2(rhi::mng::DeviceManager &device, VmaAllocator vma_allocator, VkImage image, VmaAllocation allocation, extent<2,u32> ext);
      ~VK_Image2();

    private:
      rhi::mng::DeviceManager &m_device;
      VmaAllocator m_vma_allocator{nil};
      VmaAllocation m_allocation{nil};
      VkImage m_image{nil};
      extent<2,u32> m_ext;

    public:
      fun ext() const -> extent<2,u32> override { return m_ext; }
      fun makeView() const -> sptr<ImageView2> override;
  
    public:
      fun get() -> VkImage { return m_image; }  
  };


  struct VK_ImageView2 final: public rhi::ImageView2 {
    public:
      explicit VK_ImageView2(vk::raii::ImageView imageview): m_imageview(std::move(imageview)) {}
    
    private:
      vk::raii::ImageView m_imageview;

    public:
      fun get() -> VkImageView { return *m_imageview; }
  };

}
