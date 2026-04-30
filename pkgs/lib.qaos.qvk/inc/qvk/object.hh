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
#include "qvk/types.hh"
#include <vulkan/vulkan_raii.hpp>



namespace qvk
{

  struct object
  {
    public:
      explicit object(qvk::mat4<f32> m_model, qvk::camera *camera, qvk::pipeline *pipeline, qvk::buffer *buffer, u32 count);

    public:
      static fun make(qvk::engine &engine, qvk::mat4<f32> model, qvk::camera *camera, qvk::pipeline *pipeline, qvk::buffer *buffer, u32 count) -> object*;


    private:
      qvk::mat4<f32> m_model;
      qvk::camera*   m_camera;
      qvk::pipeline* m_pipeline;
      qvk::buffer*   m_buffer;
      u32            m_count;

    public:
      inline fun& model() { return m_model; }
      inline fun  camera() { return m_camera; }
      inline fun  pipeline() { return m_pipeline; }
      inline fun  buffer() { return m_buffer; }
      inline fun  count() { return m_count; }
  };

}
