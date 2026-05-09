/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/entity/node.hh"



namespace mochi::entity
{

  /** @brief Point light object. Wraps LightComponent. */
  class Skeleton3D: public Node
  {
    public:
      explicit Skeleton3D(core &eng);
      virtual ~Skeleton3D() override = default;

      
    private:
      std::vector<mat4<f32>> m_bones;

      fun updateComponent() -> void;

    public:
      inline fun& getBones() const { return m_bones; }
  };

}
