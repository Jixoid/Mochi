/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "qvk/rhi/buffer.hh"
#include "qvk/types.hh"
#include "qvk/geometry.hh"
#include <string_view>
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace qvk
{

  /** @brief Represents a single vertex in a mesh, including position, normal, color, and UV data. */
  struct vertex_t {
    /** @brief 3D position of the vertex. */
    vec3<f32> position;
    /** @brief 3D normal vector of the vertex. */
    vec3<f32> normal;
    /** @brief RGB color of the vertex. */
    vec3<f32> color;
    /** @brief 2D texture coordinates (UV) of the vertex. */
    vec2<f32> uv;
  };
  
  /** @brief Buffer info describing the vertex_t memory layout. */
  extern info<buffer> vertex_i;
  

  

  /** @brief Represents a 3D model/mesh entity loaded into a Vulkan buffer. */
  struct mesh
  {
    public:
      /**
       * @brief Construct a mesh directly from an existing buffer.
       * @param data The buffer containing vertex data.
       */
      explicit mesh(buffer *data);
      
      /**
       * @brief Construct a mesh by loading a 3D model file from disk.
       * @param core The QVK core instance.
       * @param fpath The file path to the 3D model.
       */
      explicit mesh(core &core, std::string_view fpath);
      
    public:
      /**
       * @brief Factory method to create a mesh from an existing buffer.
       * @param core The QVK core instance.
       * @param data The buffer containing vertex data.
       * @return Pointer to the newly created mesh.
       */
      static fun make(core &core, buffer *data) -> mesh*;
      
      /**
       * @brief Factory method to load and create a mesh from a file.
       * @param core The QVK core instance.
       * @param fpath The file path to the 3D model.
       * @return Pointer to the newly created mesh.
       */
      static fun make(core &core, std::string_view fpath) -> mesh*;
      

    private:
      buffer *m_data{nil};

    public:
      /** @brief Access the underlying buffer holding the mesh vertex data. */
      inline fun data() { return m_data; }

  };

}
