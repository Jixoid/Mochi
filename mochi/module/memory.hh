/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/types.hh"
#include <cassert>
#include <functional>



namespace mochi::module
{

  struct memory
  {
    public:
      explicit memory(module::bridge &bridge, module::device &device);
      ~memory();


    private:
      module::bridge &m_bridge;
      module::device &m_device;

      vk::raii::CommandPool m_transfer_pool{nil};

      
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
      
      inline fun sharedMemory() { return m_sharedMemory; }

      inline fun pushConstant() { return m_pushConstant; }
      inline fun uniformRange() { return m_uniformRange; }
      inline fun uniformAlign() { return m_uniformAlign; }
      inline fun storageRange() { return m_storageRange; }
      inline fun storageAlign() { return m_storageAlign; }
    #pragma endregion


    
    private:
      sptr<rhi::buffer> m_camera_ubo;
      sptr<rhi::buffer> m_light_ubo;

    public:
      fun camera_ubo(u64 required_count = 0) -> sptr<rhi::buffer>;
      fun light_ubo(u64 required_count = 0) -> sptr<rhi::buffer>;



    #pragma region Allocation
    private:
      fun load_UMA_UniformBuffer(sptr<rhi::info<rhi::buffer>> info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>;
      fun load_UMA_StorageBuffer(sptr<rhi::info<rhi::buffer>> info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>;
      fun load_UMA_VertexBuffer(sptr<rhi::info<rhi::buffer>> info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>;

      fun load_DISC_UniformBuffer(sptr<rhi::info<rhi::buffer>> info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>;
      fun load_DISC_StorageBuffer(sptr<rhi::info<rhi::buffer>> info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>;
      fun load_DISC_VertexBuffer(sptr<rhi::info<rhi::buffer>> info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>;

    public:
      inline fun load_UniformBuffer(sptr<rhi::info<rhi::buffer>> info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer> { return m_sharedMemory ? load_UMA_UniformBuffer(info, count, data) : load_DISC_UniformBuffer(info, count, data); }
      inline fun load_StorageBuffer(sptr<rhi::info<rhi::buffer>> info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer> { return m_sharedMemory ? load_UMA_StorageBuffer(info, count, data) : load_DISC_StorageBuffer(info, count, data); }
      inline fun load_VertexBuffer(sptr<rhi::info<rhi::buffer>> info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer> { return m_sharedMemory ? load_UMA_VertexBuffer(info, count, data) : load_DISC_VertexBuffer(info, count, data); }
    #pragma endregion

  };

}
