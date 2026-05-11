/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/rhi/rhi.hh"
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
    requires (std::is_same_v<T, BufferCreate>)
  inline fun VKConvert(BufferCreate in) -> VmaAllocationCreateFlagBits
  {
    return static_cast<VmaAllocationCreateFlagBits>(in);
  }


  template <typename T>
    requires (std::is_same_v<T, BufferCreateFlags>)
  inline fun VKConvert(BufferCreateFlags in) -> VmaAllocationCreateFlags
  {
    return static_cast<VmaAllocationCreateFlags>(in.mask());
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
    requires (std::is_same_v<T, Format>)
  inline fun VKConvert(Format in) -> vk::Format
  {
    return static_cast<vk::Format>(in);
  }

}
