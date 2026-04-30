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
#include "qvk/meta.hh"
#include "qvk/types.hh"
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace qvk
{

  struct camera
  {
    public:
      explicit camera(qvk::engine &engine, vec3<f32> pos, vec3<f32> target, vec3<f32> up, f32 fov, f32 near, f32 far);
      explicit camera(qvk::engine &engine, vec3<f32> pos, quaternion<f32> rot, f32 fov, f32 near, f32 far);

    public:
      static fun make(qvk::engine &engine, vec3<f32> pos, vec3<f32> target, vec3<f32> up, f32 fov, f32 near, f32 far, metaobj<camera> *meta = Nil) -> camera*;
      static fun make(qvk::engine &engine, vec3<f32> pos, quaternion<f32> rot, f32 fov, f32 near, f32 far, metaobj<camera> *meta = Nil) -> camera*;


    public:
      static fun meta(qvk::metac &meta, f32 priority = 1.0) -> metaid;


    private:
      qvk::window &m_window;

      vec3<f32> m_pos;
      quaternion<f32> m_rot;

      f32 m_fov, m_near, m_far;
      mat4<f32> m_proj, m_view;

    public:
      inline fun& pos() { return m_pos; }
      inline fun& rot() { return m_rot; }
      
      inline fun& fov() { return m_fov; }
      inline fun& near() { return m_near; }
      inline fun& far() { return m_far; }
      
      inline fun proj() { return m_proj; }
      inline fun view() { return m_view; }

    public:
      fun recalc() -> void;

  };

}
