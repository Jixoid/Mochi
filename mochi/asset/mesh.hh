/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/asset/material.hh"
#include "mochi/basis.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/types.hh"
#include <span>
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace mochi::asset
{

  /** @brief Represents a single vertex in a mesh, including position, normal, color, and UV data. */
  struct vertex_t {
    vec3<f32> position;
    vec3<f32> normal;
    vec3<f32> color;
    vec2<f32> uv;
  };
  
  /** @brief Buffer info describing the vertex_t memory layout. */
  extern sptr<rhi::info<rhi::buffer>> vertex_i;


  

  /** @brief Represents a 3D model/mesh entity loaded into a Vulkan buffer. */
  struct mesh
  {
    public:
      /**
       * @brief Construct a mesh directly from an existing buffer.
       * @param data The buffer containing vertex data.
       */
      explicit mesh(sptr<rhi::buffer> data, std::vector<offs> offs, std::vector<sptr<asset::material>> material, std::vector<int> map);
      
      explicit mesh(core &core, std::span<char> file, std::string_view ext);
      
    public:
      /**
       * @brief Factory method to create a mesh from an existing buffer.
       * @param core The mochi core instance.
       * @param data The buffer containing vertex data.
       * @return Pointer to the newly created mesh.
       */
      static inline fun make(sptr<rhi::buffer> data, std::vector<offs> offs, std::vector<sptr<asset::material>> material, std::vector<int> map) -> sptr<mesh> {
        return make_sptr<mesh>(data, offs, material, map);
      }
      
      /**
       * @brief Factory method to load and create a mesh from a file.
       * @param core The mochi core instance.
       * @param fpath The file path to the 3D model.
       * @return Pointer to the newly created mesh.
       */
      static inline fun make(core &core, std::span<char> file, std::string_view ext) -> sptr<mesh> {
        return make_sptr<mesh>(core, file, ext);
      }
      

    private:
      sptr<rhi::buffer> m_data;
      std::vector<offs> m_offs;
      std::vector<sptr<asset::material>> m_material;
      std::vector<int>  m_map;

    public:
      /** @brief Access the underlying buffer holding the mesh vertex data. */
      inline fun& data() { return m_data; }
      inline fun& offs() { return m_offs; }
      inline fun& material() { return m_material; }
      inline fun& map() { return m_map; }

  };

}
