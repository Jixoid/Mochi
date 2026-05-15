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
  };

}
