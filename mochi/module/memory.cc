/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/module/memory.hh"
#include "mochi/ecs/camera.hh"
#include "mochi/ecs/point_light.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/rhi/rhi.hh"
#include "mochi/types.hh"



namespace mochi::module
{

  memory::memory(rhi::DeviceManager &device, sptr<rhi::AllocManager> alloc_mgr)
    : m_device(device), m_alloc_mgr(alloc_mgr)
  {
    // Hardcoded minimums to remove Vulkan dependency
    m_pushConstant = 128;
    m_uniformRange = 65536;
    m_uniformAlign = 256;
    m_storageRange = 134217728;
    m_storageAlign = 256;
    m_allocCount = 4096;
    m_mapAlign   = 64;
    m_vramSize = 1024 * 1024 * 1024; // 1GB
    m_sharedMemory = false;
  }

  memory::~memory()
  {
    m_camera_ubo.reset();
    m_light_ubo.reset();
  }

  


  fun memory::camera_ubo(u64 required_count) -> sptr<rhi::Buffer>
  {
    if (!m_camera_ubo || (m_camera_ubo->size() / sizeof(ecs::camera_t)) < required_count)
      m_camera_ubo = m_alloc_mgr->allocBuffer(
        sizeof(ecs::camera_t) * std::max<u64>(10, required_count),
        rhi::BufferUsage::UniformBuffer, 
        flags(rhi::AllocationCreate::HostSequentialWrite) | rhi::AllocationCreate::Mapped,
        rhi::AllocationLocation::PreferHost
      );

    return m_camera_ubo;
  }

  fun memory::light_ubo(u64 required_count) -> sptr<rhi::Buffer>
  {
    if (!m_light_ubo || (m_light_ubo->size() / sizeof(ecs::point_light_t)) < required_count)
      m_light_ubo = m_alloc_mgr->allocBuffer(
        sizeof(ecs::point_light_t) * std::max<u64>(100, required_count),
        rhi::BufferUsage::UniformBuffer, 
        flags(rhi::AllocationCreate::HostSequentialWrite) | rhi::AllocationCreate::Mapped,
        rhi::AllocationLocation::PreferHost
      );

    return m_light_ubo;
  }

}
