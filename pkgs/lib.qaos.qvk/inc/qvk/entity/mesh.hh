/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "qvk/entity/buffer.hh"
#include "qvk/types.hh"
#include "qvk/geometry.hh"
#include <string_view>
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace qvk
{

  struct vertex_t {
    vec3<f32> position;
    vec3<f32> normal;
    vec3<f32> color;
    vec2<f32> uv;
  };
  extern info<buffer> vertex_i;
  


  struct mesh
  {
    public:
      explicit mesh(buffer *data);
      explicit mesh(core &core, std::string_view fpath);
      
    public:
      static fun make(core &core, buffer *data) -> mesh*;
      static fun make(core &core, std::string_view fpath) -> mesh*;
      

    private:
      buffer *m_data{nullptr};

    public:
      inline fun data() { return m_data; }

  };

}
