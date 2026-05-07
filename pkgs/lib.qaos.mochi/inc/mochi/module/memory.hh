/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/types.hh"
#include <cassert>
#include <functional>
#include <vulkan/vulkan.hpp>
#include <vulkan/vulkan_raii.hpp>
#include "vk_mem_alloc.h"



namespace mochi::module
{

  /** @brief Manages Vulkan memory allocation, device limits, and entity ownership. */
  struct memory
  {
    public:
      /**
       * @brief Initialize the memory allocator.
       * @param bridge The mochi vulkan bridge.
       * @param device The logical device.
       * @param renderer The renderer.
       */
      explicit memory(module::bridge &bridge, module::device &device, module::renderer &renderer);

      /** @brief Destructor. */
      ~memory();


    private:
      module::bridge &m_bridge;
      module::device &m_device;
      module::renderer &m_renderer;

      
    #pragma region Limits
    private:
      VmaAllocator m_allocator{nil};

      bool m_sharedMemory{};

      u32 m_pushConstant;
      u64 m_uniformRange;
      u32 m_uniformAlign;
      u64 m_storageRange;
      u32 m_storageAlign;
      u64 m_allocCount;
      u64 m_mapAlign;
      u64 m_vramSize;


    public:
      /** @brief Get the vma handle. */
      inline fun allocator() { return m_allocator; }
      
      /** @brief Check if the device uses Unified Memory Architecture (UMA). */
      inline fun sharedMemory() { return m_sharedMemory; }

      /** @brief Get the maximum push constant size. */
      inline fun pushConstant() { return m_pushConstant; }
      /** @brief Get the uniform buffer range limit. */
      inline fun uniformRange() { return m_uniformRange; }
      /** @brief Get the uniform buffer alignment requirement. */
      inline fun uniformAlign() { return m_uniformAlign; }
      /** @brief Get the storage buffer range limit. */
      inline fun storageRange() { return m_storageRange; }
      /** @brief Get the storage buffer alignment requirement. */
      inline fun storageAlign() { return m_storageAlign; }
    #pragma endregion


    
    public:
      sptr<rhi::buffer> m_camera_ubo;
      sptr<rhi::buffer> m_light_ubo;



    #pragma region Allocation
    private:
      fun load_UMA_UniformBuffer(rhi::info<rhi::buffer> *info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>;
      fun load_UMA_StorageBuffer(rhi::info<rhi::buffer> *info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>;
      fun load_UMA_VertexBuffer(rhi::info<rhi::buffer> *info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>;

      fun load_DISC_UniformBuffer(rhi::info<rhi::buffer> *info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>;
      fun load_DISC_StorageBuffer(rhi::info<rhi::buffer> *info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>;
      fun load_DISC_VertexBuffer(rhi::info<rhi::buffer> *info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>;

    public:
      /** @brief Allocate and optionally initialize a uniform buffer (handles UMA vs Discrete). */
      inline fun load_UniformBuffer(rhi::info<rhi::buffer> *info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer> { return m_sharedMemory ? load_UMA_UniformBuffer(info, count, data) : load_DISC_UniformBuffer(info, count, data); }
      /** @brief Allocate and optionally initialize a storage buffer (handles UMA vs Discrete). */
      inline fun load_StorageBuffer(rhi::info<rhi::buffer> *info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer> { return m_sharedMemory ? load_UMA_StorageBuffer(info, count, data) : load_DISC_StorageBuffer(info, count, data); }
      /** @brief Allocate and optionally initialize a vertex buffer (handles UMA vs Discrete). */
      inline fun load_VertexBuffer(rhi::info<rhi::buffer> *info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer> { return m_sharedMemory ? load_UMA_VertexBuffer(info, count, data) : load_DISC_VertexBuffer(info, count, data); }
    #pragma endregion

  };

}
