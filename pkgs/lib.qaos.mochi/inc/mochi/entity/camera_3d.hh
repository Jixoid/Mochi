/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/entity/node_3d.hh"



namespace mochi::entity
{

  /** @brief Camera 3D object. Wraps CameraComponent. */
  class Camera3D: public Node3D
  {
    public:
      explicit Camera3D(core &eng);
      virtual ~Camera3D() override = default;

    private:
      f32 m_fov{60};
      f32 m_near{0.1};
      f32 m_far{1000};

      fun updateComponent() -> void;

    public:
      inline fun getFov() const { return m_fov; }
      fun setFov(f32 fov) -> void;

      inline fun getNear() const { return m_near; }
      fun setNear(f32 n) -> void;

      inline fun getFar() const { return m_far; }
      fun setFar(f32 f) -> void;
  };

}
