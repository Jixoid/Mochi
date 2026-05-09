/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/entity/node.hh"
#include "mochi/geometry.hh"
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

      fun updateTransform() -> void;

    public:
      inline fun getPosition() const { return m_position; }
      fun setPosition(const vec3<f32> &pos) -> void;

      inline fun getRotation() const { return m_rotation; }
      fun setRotation(const quaternion<f32> &rot) -> void;

      inline fun getScale() const { return m_scale; }
      fun setScale(const vec3<f32> &scale) -> void;
  };

}
