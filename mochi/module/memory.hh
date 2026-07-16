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
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/rhi/manager/alloc_manager.hh"
#include "mochi/types.hh"
#include <cassert>



namespace mochi::module
{

  struct memory
  {
    public:
      explicit memory(rhi::DeviceManager &device, sptr<rhi::AllocManager> alloc_mgr);
      ~memory();


    private:
      rhi::DeviceManager &m_device;
      sptr<rhi::AllocManager> m_alloc_mgr;

      
    #pragma region Limits
    private:
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
      inline fun sharedMemory() { return m_sharedMemory; }

      inline fun pushConstant() { return m_pushConstant; }
      inline fun uniformRange() { return m_uniformRange; }
      inline fun uniformAlign() { return m_uniformAlign; }
      inline fun storageRange() { return m_storageRange; }
      inline fun storageAlign() { return m_storageAlign; }
    #pragma endregion


    
    private:
      sptr<rhi::Buffer> m_camera_ubo;
      sptr<rhi::Buffer> m_light_ubo;

    public:
      fun camera_ubo(u64 required_count = 0) -> sptr<rhi::Buffer>;
      fun light_ubo(u64 required_count = 0) -> sptr<rhi::Buffer>;
  };

}
