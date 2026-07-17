/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/manager/scene_manager.hh"
#include "mochi/ecs/camera.hh"
#include "mochi/ecs/point_light.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/types.hh"



namespace mochi::manager
{

  SceneManager::SceneManager(rhi::DeviceManager &dmng, rhi::AllocManager &amng)
    : m_dmng(dmng), m_amng(amng)
  {}

  SceneManager::~SceneManager() {
    m_camera_ubo.reset();
    m_light_ubo.reset();
  }



  fun SceneManager::camera_ubo(u64 required_count) -> sptr<rhi::Buffer> {
    if (!m_camera_ubo || (m_camera_ubo->size() / sizeof(ecs::camera_t)) < required_count)
      m_camera_ubo = m_amng.allocBuffer(
        sizeof(ecs::camera_t) * std::max<u64>(10, required_count),
        rhi::BufferUsage::UniformBuffer | rhi::BufferUsage::DeviceAddress, 
        rhi::AllocationCreate::HostSequentialWrite | rhi::AllocationCreate::Mapped,
        rhi::AllocationLocation::PreferHost
      );

    return m_camera_ubo;
  }

  fun SceneManager::light_ubo(u64 required_count) -> sptr<rhi::Buffer> {
    if (!m_light_ubo || (m_light_ubo->size() / sizeof(ecs::point_light_t)) < required_count)
      m_light_ubo = m_amng.allocBuffer(
        sizeof(ecs::point_light_t) * std::max<u64>(100, required_count),
        rhi::BufferUsage::UniformBuffer | rhi::BufferUsage::DeviceAddress, 
        rhi::AllocationCreate::HostSequentialWrite | rhi::AllocationCreate::Mapped,
        rhi::AllocationLocation::PreferHost
      );

    return m_light_ubo;
  }

}
