/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "qvk/entity/node.hh"
#include "qvk/geometry.hh"
#include "qvk/types.hh"
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace qvk
{

  struct light_t
  {
    vec4<f32> position; // X, Y, Z pozisyon.
    vec4<f32> color;    // R, G, B renk. W = Işık Şiddeti
  };
  extern info<buffer> light_i;


  
  struct light: node
  {
    public:
      explicit light(core &core, node *parent, mat4<f32> model, vec3<f32> color, f32 intensity);
      ~light();
      

    private:
      memory &m_memory;

      vec3<f32> m_color;
      f32 m_intensity;

    public:
      inline fun getColor() { return m_color; }
      inline fun getIntensity() { return m_intensity; }

  };

}
