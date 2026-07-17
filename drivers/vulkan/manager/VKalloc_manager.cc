/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "drivers/vulkan/manager/VKalloc_manager.hh"
#include "drivers/vulkan/manager/VKdevice_manager.hh"
#include "mochi/basis.hh"
#include "drivers/vulkan/VKconvert.hh"
#include "drivers/vulkan/manager/VKalloc_manager.hh"
#include "drivers/vulkan/VKbuffer.hh"
#include "drivers/vulkan/VKimage.hh"
#include "drivers/vulkan/VKsampler.hh"
#include "mochi/except.hh"
#include "mochi/math/extent.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/rhi/image.hh"
#include "mochi/rhi/manager/alloc_manager.hh"
#include "mochi/rhi/sampler.hh"
#include <vulkan/vulkan_raii.hpp>



namespace mochi::rhi::vulkan
{

  extern "C" fun MochiRHI_MakeAllocManager(rhi::DeviceManager &device) -> AllocManager* {
    return new VK_AllocManager(device);
  }



  VK_AllocManager::VK_AllocManager(rhi::DeviceManager &dmng)
    : rhi::AllocManager(dmng)
  {
    VmaAllocatorCreateInfo allocatorInfo = {};
    allocatorInfo.vulkanApiVersion = VK_API_VERSION_1_4;
    auto& vk_dmng = static_cast<VK_DeviceManager&>(dmng);
    allocatorInfo.physicalDevice = *vk_dmng.phys_dev();
    allocatorInfo.device = *vk_dmng.get();
    allocatorInfo.instance = *vk_dmng.inst();
    allocatorInfo.flags = VMA_ALLOCATOR_CREATE_BUFFER_DEVICE_ADDRESS_BIT;

    vmaCreateAllocator(&allocatorInfo, &vma_allocator);
  } 
  
  VK_AllocManager::~VK_AllocManager() {
    vmaDestroyAllocator(vma_allocator);
  }


      
  fun VK_AllocManager::allocBuffer(
    u64 size, BufferUsageFlags usage, AllocationCreateFlags create, AllocationLocation location
  ) -> sptr<Buffer>
  {
    VmaAllocationCreateInfo alloc_info{};
    alloc_info.usage = VKConvert<AllocationLocation>(location);
    alloc_info.flags = VKConvert<AllocationCreateFlags>(create);
    if ((alloc_info.flags & VMA_ALLOCATION_CREATE_MAPPED_BIT) != 0 && 
        (alloc_info.flags & VMA_ALLOCATION_CREATE_HOST_ACCESS_RANDOM_BIT) == 0) {
      alloc_info.flags |= VMA_ALLOCATION_CREATE_HOST_ACCESS_SEQUENTIAL_WRITE_BIT;
    }

    vk::BufferCreateInfo buffer_create_info{
      {},
      size,
      VKConvert<BufferUsageFlags>(usage),
      vk::SharingMode::eExclusive
    };
    
    VmaAllocationInfo vma_alloc_info{};

    VkBuffer e_buffer{};
    VmaAllocation e_allocation{};
    void* e_addr{};

    auto res = vmaCreateBuffer(
      vma_allocator,
      buffer_create_info,
      &alloc_info,
      &e_buffer,
      &e_allocation,
      &vma_alloc_info
    );

    if (res != VK_SUCCESS)
      throw mochi::rhi_error("Failed to create VMA Buffer! Error code: " + std::to_string(res));
    
    if (create & AllocationCreate::Mapped)
      e_addr = vma_alloc_info.pMappedData;

    
    return make_sptr(new VK_Buffer(m_dmng, vma_allocator, e_buffer, e_allocation, e_addr, size));
  }
  
  
  fun VK_AllocManager::allocImage2(
    extent<2,u32> ext, Format format, ImageUsageFlags usage, ImageTiling tiling,
    AllocationCreateFlags create, AllocationLocation location
  ) -> sptr<Image2>
  {
    VmaAllocationCreateInfo alloc_info = {};
    alloc_info.usage = VKConvert<AllocationLocation>(location);
    alloc_info.flags = VKConvert<AllocationCreateFlags>(create);
    if ((alloc_info.flags & VMA_ALLOCATION_CREATE_MAPPED_BIT) != 0 && 
        (alloc_info.flags & VMA_ALLOCATION_CREATE_HOST_ACCESS_RANDOM_BIT) == 0) {
      alloc_info.flags |= VMA_ALLOCATION_CREATE_HOST_ACCESS_SEQUENTIAL_WRITE_BIT;
    }

    vk::ImageCreateInfo image_info(
      {}, vk::ImageType::e2D, VKConvert<Format>(format),
      vk::Extent3D(ext.x(), ext.y(), 1),
      1, 1, vk::SampleCountFlagBits::e1, VKConvert<ImageTiling>(tiling),
      VKConvert<ImageUsageFlags>(usage) | vk::ImageUsageFlagBits::eHostTransfer,
      vk::SharingMode::eExclusive
    );

    VkImage e_image;
    VmaAllocation e_allocation;

    auto res = vmaCreateImage(
      vma_allocator, 
      image_info, 
      &alloc_info, 
      &e_image,
      &e_allocation,
      nullptr
    );

    if (res != VK_SUCCESS)
      throw mochi::rhi_error("Failed to create VMA Buffer! Error code: " + std::to_string(res));


    return make_sptr(new VK_Image2(m_dmng, vma_allocator, e_image, e_allocation, ext));
  }


  fun VK_AllocManager::allocSampler2(
    extent<3, SamplerAddressMode> addressMode, SamplerFilter magFilter, SamplerFilter minFilter
  ) -> sptr<Sampler2>
  {
    vk::SamplerCreateInfo sampler_info(
      {}, 

      VKConvert<SamplerFilter>(magFilter),
      VKConvert<SamplerFilter>(minFilter),
      
      vk::SamplerMipmapMode::eLinear,

      VKConvert<SamplerAddressMode>(addressMode.x()),
      VKConvert<SamplerAddressMode>(addressMode.y()),
      VKConvert<SamplerAddressMode>(addressMode.z()),
      
      0, VK_TRUE, 16,
      VK_FALSE, vk::CompareOp::eAlways, 0, 0, 
      vk::BorderColor::eIntOpaqueBlack, VK_FALSE
    );
    auto e_sampler = vk::raii::Sampler(static_cast<VK_DeviceManager&>(m_dmng).get(), sampler_info);


    return make_sptr(new VK_Sampler2(m_dmng, std::move(e_sampler)));
  }

}
