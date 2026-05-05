/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "qvk/geometry.hh"
#include "qvk/entity/node.hh"
#include "qvk/types.hh"
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace qvk
{

  struct camera_t {
    mat4<f32> view;
    mat4<f32> proj;
  };
  extern info<buffer> camera_i;
  

  

  struct camera: node
  {
    public:
      explicit camera(core &core, node *parent, mat4<f32> model, f32 fov, f32 near, f32 far);
      ~camera();


    private:
      memory &m_memory;
      window &m_window;

      f32 m_fov, m_near, m_far;
      mat4<f32> m_view, m_proj;

    public:
      inline fun& fov() { return m_fov; }
      inline fun& near() { return m_near; }
      inline fun& far() { return m_far; }
      
      inline fun view() { return m_view; }
      inline fun proj() { return m_proj; }
      

    public:
      fun recalc() -> void;

      inline fun setModel(mat4<f32> val) -> void override {
        node::setModel(val);
        recalc();
      }

  };

}
