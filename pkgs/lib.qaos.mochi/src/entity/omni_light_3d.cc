/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/entity/omni_light_3d.hh"
#include "mochi/world/components.hh"



namespace mochi::entity
{

  OmniLight3D::OmniLight3D(core &eng): Node3D(eng)
  {
    m_core.registry().emplace<LightComponent>(m_entity);
    update_component();
  }

  fun OmniLight3D::update_component() -> void
  {
    auto& lc = m_core.registry().get<LightComponent>(m_entity);
    lc.color = m_color;
    lc.intensity = m_intensity;
  }

  fun OmniLight3D::set_color(const vec3<f32> &color) -> void
  {
    m_color = color;
    update_component();
  }

  fun OmniLight3D::set_intensity(f32 intensity) -> void
  {
    m_intensity = intensity;
    update_component();
  }

}
