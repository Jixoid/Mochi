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

  /** @brief Point light object. Wraps LightComponent. */
  class OmniLight3D: public Node3D
  {
    public:
      explicit OmniLight3D(core &eng);
      virtual ~OmniLight3D() override = default;

      
    private:
      vec3<f32> m_color{1,1,1};
      f32 m_intensity{10};

      fun update_component() -> void;

    public:
      inline fun get_color() const { return m_color; }
      inline fun get_intensity() const { return m_intensity; }

      fun set_color(const vec3<f32> &color) -> void;
      fun set_intensity(f32 intensity) -> void;
  };

}
