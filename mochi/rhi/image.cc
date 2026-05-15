/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/rhi/rhi.hh"
#include "mochi/types.hh"
#include "mochi/rhi/image.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/module/memory.hh"
#include "mochi/module/device.hh"
#include <vulkan/vulkan_raii.hpp>
#include "vk_mem_alloc.h"



namespace mochi::rhi
{

  image2::image2(module::device &device, module::memory &memory, u32 width, u32 height, void *ptr)
    : m_width(width)
    , m_height(height)
  {
    vk::DeviceSize image_size = width * height * 4;

    auto staging_info = rhi::info<rhi::buffer>::make(1, {}); 
    
    VmaAllocationCreateInfo staging_alloc_info = {};
    staging_alloc_info.usage = VMA_MEMORY_USAGE_AUTO;
    staging_alloc_info.flags = VMA_ALLOCATION_CREATE_MAPPED_BIT | VMA_ALLOCATION_CREATE_HOST_ACCESS_SEQUENTIAL_WRITE_BIT;

    auto staging_buffer = rhi::buffer::make(
      device, memory, staging_info, image_size,
      BufferUsage::TransferSrc,
      flags(BufferCreate::Mapped) | BufferCreate::HostSequentialWrite,
      BufferLocation::PreferHost
    );

    std::memcpy(staging_buffer->mapped(), ptr, static_cast<size_t>(image_size));

    vk::ImageCreateInfo image_info(
      {}, vk::ImageType::e2D, vk::Format::eR8G8B8A8Srgb,
      vk::Extent3D(width, height, 1),
      1, 1, vk::SampleCountFlagBits::e1, vk::ImageTiling::eOptimal,
      vk::ImageUsageFlagBits::eTransferDst | vk::ImageUsageFlagBits::eSampled,
      vk::SharingMode::eExclusive
    );
    
    VmaAllocationCreateInfo image_alloc_info = {};
    image_alloc_info.usage = VMA_MEMORY_USAGE_AUTO_PREFER_DEVICE;


    VkImageCreateInfo c_image_info = static_cast<VkImageCreateInfo>(image_info);
    
    vmaCreateImage(
      memory.allocator(), 
      &c_image_info, 
      &image_alloc_info, 
      &m_image, 
      &m_allocation, 
      nullptr
    );


    auto &cmd = device.transferBuf();
    
    vk::ImageMemoryBarrier barrier_to_transfer(
      vk::AccessFlagBits::eNone, vk::AccessFlagBits::eTransferWrite,
      vk::ImageLayout::eUndefined, vk::ImageLayout::eTransferDstOptimal,
      VK_QUEUE_FAMILY_IGNORED, VK_QUEUE_FAMILY_IGNORED,
      m_image, {vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1}
    );
    
    cmd.pipelineBarrier(
      vk::PipelineStageFlagBits::eTopOfPipe, vk::PipelineStageFlagBits::eTransfer,
      {}, {}, {}, {barrier_to_transfer}
    );


    vk::BufferImageCopy copy_region(
      0, 0, 0,
      {vk::ImageAspectFlagBits::eColor, 0, 0, 1},
      {0, 0, 0}, {width, height, 1}
    );
    

    cmd.copyBufferToImage(staging_buffer->get(), m_image, vk::ImageLayout::eTransferDstOptimal, {copy_region});
    device.addTransferBufRef(staging_buffer);


    vk::ImageMemoryBarrier barrier_to_shader(
      vk::AccessFlagBits::eTransferWrite, vk::AccessFlagBits::eShaderRead,
      vk::ImageLayout::eTransferDstOptimal, vk::ImageLayout::eShaderReadOnlyOptimal,
      VK_QUEUE_FAMILY_IGNORED, VK_QUEUE_FAMILY_IGNORED,
      m_image, {vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1}
    );

    cmd.pipelineBarrier(
      vk::PipelineStageFlagBits::eTransfer, vk::PipelineStageFlagBits::eFragmentShader,
      {}, {}, {}, {barrier_to_shader}
    );


    vk::ImageViewCreateInfo view_info(
      {}, m_image, vk::ImageViewType::e2D, vk::Format::eR8G8B8A8Srgb,
      {}, {vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1}
    );
    m_view = vk::raii::ImageView(device.vdevice(), view_info);

    vk::SamplerCreateInfo sampler_info(
      {}, 
      vk::Filter::eLinear, vk::Filter::eLinear, vk::SamplerMipmapMode::eLinear,
      vk::SamplerAddressMode::eRepeat, vk::SamplerAddressMode::eRepeat, vk::SamplerAddressMode::eRepeat,
      0.0f, VK_TRUE, 16.0f,
      VK_FALSE, vk::CompareOp::eAlways, 0.0f, 0.0f, 
      vk::BorderColor::eIntOpaqueBlack, VK_FALSE
    );
    m_sampler = vk::raii::Sampler(device.vdevice(), sampler_info);
  }

  image2::~image2()
  {
    if (m_image)
      vmaDestroyImage(m_allocator, m_image, m_allocation);
  }

}
