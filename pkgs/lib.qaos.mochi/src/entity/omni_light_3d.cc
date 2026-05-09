/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/entity/omni_light_3d.hh"
#include "mochi/ecs/omni_light_3d.hh"
#include "mochi/core.hh"



namespace mochi::entity
{

  OmniLight3D::OmniLight3D(core &eng): Node3D(eng)
  {
    m_core.registry().emplace<ecs::OmniLight3D>(m_entity);
    updateComponent();
  }

  fun OmniLight3D::updateComponent() -> void
  {
    auto& lc = m_core.registry().get<ecs::OmniLight3D>(m_entity);
    lc.color = m_color;
    lc.intensity = m_intensity;
  }

  fun OmniLight3D::setColor(const vec3<f32> &color) -> void
  {
    m_color = color;
    updateComponent();
  }

  fun OmniLight3D::setIntensity(f32 intensity) -> void
  {
    m_intensity = intensity;
    updateComponent();
  }

}
