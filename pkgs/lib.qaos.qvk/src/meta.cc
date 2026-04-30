/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "Basis.h"
#include "Basis.hh"
#include "qvk/device.hh"
#include "qvk/types.hh"
#include "qvk/meta.hh"
#include <cassert>
#include <format>
#include <iostream>
#include <string>
#include <vector>
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace qvk
{

  inline fun getSize(u0 __size) -> std::string
  {
    f32 size = __size;

    std::vector<std::string> typs = {"B", "KB", "MB", "GB", "TB", "PB"};
    int idx{};

    while (size >= 1024) {
      idx++;
      size /= 1024;
    }

    return std::format("{}{}", size, typs[idx]);
  }

  

  meta::meta(qvk::device &device)
    : m_device(device)
  {
    vk::PhysicalDeviceProperties props = m_device.phys_dev().getProperties();
    vk::PhysicalDeviceMemoryProperties mem_props = m_device.phys_dev().getMemoryProperties();
    


    std::cout << "Push Constant: " << getSize(props.limits.maxPushConstantsSize) << std::endl;
    
    std::cout << "UBO Aling: " << getSize(props.limits.minUniformBufferOffsetAlignment) << std::endl;
    std::cout << "UBO Range: " << getSize(props.limits.maxUniformBufferRange) << std::endl;

    std::cout << "SSBO Align: " << getSize(props.limits.minStorageBufferOffsetAlignment) << std::endl;
    std::cout << "SSBO Range: " << getSize(props.limits.maxStorageBufferRange) << std::endl;

    

    for (u32 i{}; i < mem_props.memoryTypeCount; i++)
      std::cerr << "vram["<<i<<"]"
        << ", HeadIndex: " << mem_props.memoryTypes[i].heapIndex
        << ", DeviceLocal: " << (bool)(mem_props.memoryTypes[i].propertyFlags & vk::MemoryPropertyFlagBits::eDeviceLocal)
        << ", HostVisible: " << (bool)(mem_props.memoryTypes[i].propertyFlags & vk::MemoryPropertyFlagBits::eHostVisible)
        << ", HostCoherent: " << (bool)(mem_props.memoryTypes[i].propertyFlags & vk::MemoryPropertyFlagBits::eHostCoherent)
        << ", HostCached: " << (bool)(mem_props.memoryTypes[i].propertyFlags & vk::MemoryPropertyFlagBits::eHostCached)
        << ", LazilyAllocated: " << (bool)(mem_props.memoryTypes[i].propertyFlags & vk::MemoryPropertyFlagBits::eLazilyAllocated)
        << ", Protected: " << (bool)(mem_props.memoryTypes[i].propertyFlags & vk::MemoryPropertyFlagBits::eProtected) << std::endl;

    
    u32 i{};
    for (auto &X: mem_props.memoryHeaps)
      if (X.flags & vk::MemoryHeapFlagBits::eDeviceLocal)
        std::cout << "vram["<<i++<<"] Size: " << getSize(X.size) << std::endl;
      
  }

}
