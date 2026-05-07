/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/entity/node3d.hh"



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

      fun update_component() -> void;

    public:
      inline fun get_fov() const { return m_fov; }
      inline fun get_near() const { return m_near; }
      inline fun get_far() const { return m_far; }

      fun set_fov(f32 fov) -> void;
      fun set_near(f32 n) -> void;
      fun set_far(f32 f) -> void;
      
      fun make_current() -> void; // Could be used later to set active camera
  };

}
