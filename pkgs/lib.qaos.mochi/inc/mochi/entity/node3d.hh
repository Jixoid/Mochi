/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/entity/node.hh"
#include "mochi/types.hh"



namespace mochi::entity
{

  /** @brief Base class for 3D spatial objects. Wraps TransformComponent. */
  class Node3D: public Node
  {
    public:
      explicit Node3D(core &eng);
      virtual ~Node3D() override = default;

      
    protected:
      vec3<f32>       m_position{0,0,0};
      quaternion<f32> m_rotation{};
      vec3<f32>       m_scale{1,1,1};

      fun update_transform() -> void;

    public:
      inline fun get_position() const { return m_position; }
      inline fun get_rotation() const { return m_rotation; }
      inline fun get_scale() const { return m_scale; }

      fun set_position(const vec3<f32> &pos) -> void;
      fun set_rotation(const quaternion<f32> &rot) -> void;
      fun set_scale(const vec3<f32> &scale) -> void;
  };

}
