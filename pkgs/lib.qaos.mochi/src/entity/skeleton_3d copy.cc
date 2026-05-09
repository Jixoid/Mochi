/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/entity/skeleton_3d.hh"
#include "mochi/ecs/skeleton_3d.hh"
#include "mochi/core.hh"



namespace mochi::entity
{

  Skeleton3D::Skeleton3D(core &eng): Node(eng)
  {
    m_core.registry().emplace<ecs::Skeleton3D>(m_entity);
    updateComponent();
  }

  fun Skeleton3D::updateComponent() -> void
  {
    auto& lc = m_core.registry().get<ecs::Skeleton3D>(m_entity);
    lc.bones = m_bones;
  }

}
