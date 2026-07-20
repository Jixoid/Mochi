/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once


#include "mochi/basis.hh"
#include "mochi/rhi/manager/allocator.hh"
#include "mochi/rhi/image.hh"
#include "mochi/rhi/pipeline.hh"
#include "mochi/rhi/types.hh"
#include "mochi/rhi/shader.hh"
#include "mochi/rhi/buffer.hh"
#include <type_traits>
#include <vulkan/vulkan.hpp>
#include "vk_mem_alloc.h"



namespace mochi::rhi
{

  template <typename>
  fun VKConvert();
  


  template <typename T>
    requires (std::is_same_v<T, ShaderStage>)
  inline fun VKConvert(ShaderStage in) -> vk::ShaderStageFlagBits
  {
    return static_cast<vk::ShaderStageFlagBits>(in);
  }


  template <typename T>
    requires (std::is_same_v<T, ShaderStageFlags>)
  inline fun VKConvert(ShaderStageFlags in) -> vk::ShaderStageFlags
  {
    return static_cast<vk::ShaderStageFlags>(in.mask());
  }


  template <typename T>
    requires (std::is_same_v<T, BufferUsage>)
  inline fun VKConvert(BufferUsage in) -> vk::BufferUsageFlagBits
  {
    return static_cast<vk::BufferUsageFlagBits>(in);
  }


  template <typename T>
    requires (std::is_same_v<T, BufferUsageFlags>)
  inline fun VKConvert(BufferUsageFlags in) -> vk::BufferUsageFlags
  {
    return static_cast<vk::BufferUsageFlags>(in.mask());
  }


  template <typename T>
    requires (std::is_same_v<T, ImageUsage>)
  inline fun VKConvert(ImageUsage in) -> vk::ImageUsageFlagBits
  {
    return static_cast<vk::ImageUsageFlagBits>(in);
  }


  template <typename T>
    requires (std::is_same_v<T, ImageUsageFlags>)
  inline fun VKConvert(ImageUsageFlags in) -> vk::ImageUsageFlags
  {
    return static_cast<vk::ImageUsageFlags>(in.mask());
  }


  template <typename T>
    requires (std::is_same_v<T, ImageTiling>)
  inline fun VKConvert(ImageTiling in) -> vk::ImageTiling
  {
    return static_cast<vk::ImageTiling>(in);
  }


  template <typename T>
    requires (std::is_same_v<T, SamplerAddressMode>)
  inline fun VKConvert(SamplerAddressMode in) -> vk::SamplerAddressMode
  {
    return static_cast<vk::SamplerAddressMode>(in);
  }


  template <typename T>
    requires (std::is_same_v<T, SamplerFilter>)
  inline fun VKConvert(SamplerFilter in) -> vk::Filter
  {
    return static_cast<vk::Filter>(in);
  }


  template <typename T>
    requires (std::is_same_v<T, rhi::AllocationCreate>)
  inline fun VKConvert(rhi::AllocationCreate in) -> VmaAllocationCreateFlagBits
  {
    return static_cast<VmaAllocationCreateFlagBits>(in);
  }


  template <typename T>
    requires (std::is_same_v<T, rhi::AllocationCreateFlags>)
  inline fun VKConvert(rhi::AllocationCreateFlags in) -> VmaAllocationCreateFlags
  {
    return static_cast<VmaAllocationCreateFlags>(in.mask());
  }


  template <typename T>
    requires (std::is_same_v<T, rhi::AllocationLocation>)
  inline fun VKConvert(rhi::AllocationLocation in) -> VmaMemoryUsage
  {
    return static_cast<VmaMemoryUsage>(in);
  }

  
  template <typename T>
    requires (std::is_same_v<T, VertexInputRate>)
  inline fun VKConvert(VertexInputRate in) -> vk::VertexInputRate
  {
    return static_cast<vk::VertexInputRate>(in);
  }


  template <typename T>
    requires (std::is_same_v<T, DescriptorType>)
  inline fun VKConvert(DescriptorType in) -> vk::DescriptorType
  {
    return static_cast<vk::DescriptorType>(in);
  }


  template <typename T>
    requires (std::is_same_v<T, PolygonMode>)
  inline fun VKConvert(PolygonMode in) -> vk::PolygonMode
  {
    return static_cast<vk::PolygonMode>(in);
  }


  template <typename T>
    requires (std::is_same_v<T, PrimitiveTopology>)
  inline fun VKConvert(PrimitiveTopology in) -> vk::PrimitiveTopology
  {
    return static_cast<vk::PrimitiveTopology>(in);
  }


  template <typename T>
    requires (std::is_same_v<T, PipelineKind>)
  inline fun VKConvert(PipelineKind in) -> vk::PipelineBindPoint
  {
    return static_cast<vk::PipelineBindPoint>(in);
  }


  template <typename T>
    requires (std::is_same_v<T, Format>)
  inline fun VKConvert(Format in) -> vk::Format
  {
    return static_cast<vk::Format>(in);
  }

  template <typename T>
    requires (std::is_same_v<T, Format>)
  inline fun VKDeConvert(vk::Format in) -> Format
  {
    return static_cast<Format>(in);
  }

}
