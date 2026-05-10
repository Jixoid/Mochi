/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/asset/mesh.hh"
#include "mochi/module/memory.hh"
#include "mochi/reader/reader.hh"
#include "mochi/types.hh"
#include "mochi/core.hh"
#include <cstring>
#include <string>
#include <string_view>
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace mochi::asset
{
  
  rhi::info<rhi::buffer> vertex_i = rhi::info<rhi::buffer>(
    sizeof(vertex_t),
    vt::make_list<
      vec3<f32>, // Position
      vec3<f32>, // Normal
      vec3<f32>, // Color
      vec2<f32>  // Uv
    >()
  );




  inline fun build_vertices(wf_obj &raw_data) -> std::vector<vertex_t> 
  {
    std::vector<vertex_t> vertexs;

    for (const auto &face: raw_data.f) 
    {
      if (face.vertices.size() < 3) continue;

      for (u0 i = 1; i < face.vertices.size()-1; i++) 
      {
        wf_index tri_indices[3] = { 
          face.vertices[0], 
          face.vertices[i], 
          face.vertices[i+1] 
        };

        auto p0 = raw_data.v[tri_indices[0].v_idx];
        auto p1 = raw_data.v[tri_indices[1].v_idx];
        auto p2 = raw_data.v[tri_indices[2].v_idx];

        auto edge1 = p1-p0, edge2 = p2-p0;

        auto calculated_normal = edge1.cross(edge2).normalize();

        for (int j = 0; j < 3; j++) 
        {
          vertex_t v{};
          v.color = {1,1,1};


          if (tri_indices[j].v_idx >= 0 && tri_indices[j].v_idx < raw_data.v.size())
            v.position = raw_data.v[tri_indices[j].v_idx];


          if (tri_indices[j].vt_idx >= 0 && tri_indices[j].vt_idx < raw_data.vt.size())
            v.uv = raw_data.vt[tri_indices[j].vt_idx];
          else
            v.uv = {0.0f, 0.0f};


          if (tri_indices[j].vn_idx >= 0 && tri_indices[j].vn_idx < raw_data.vn.size())
            v.normal = raw_data.vn[tri_indices[j].vn_idx];
          else
            v.normal = calculated_normal; 
          
          vertexs.push_back(v);
        }
      }
    }

    return vertexs;
  }



  mesh::mesh(sptr<rhi::buffer> data)
    : m_data(data)
  {}

  mesh::mesh(core &core, std::string_view fpath)
  {
    mappedFile mfile((std::string)fpath);
    
    std::vector<vertex_t> final_vertices;

    if (fpath.ends_with(".obj")) {
      auto raw = read<ft_wavefront>(mfile);
      final_vertices = std::move(build_vertices(raw));
    } 
    else if (fpath.ends_with(".glb") || fpath.ends_with(".gltf")) {
      auto raw = read<ft_gltf>(mfile);
      final_vertices = std::move(raw.vertices);
    }

    m_data = core.sub<module::memory>().load_VertexBuffer(&vertex_i, final_vertices.size(),
      [&final_vertices](void* _data) {
        memcpy(_data, final_vertices.data(), vertex_i.stride() * final_vertices.size());
      }
    );
  }


  fun mesh::make(core &core, sptr<rhi::buffer> data) -> sptr<mesh>
  {
    return make_sptr<mesh>(data);
  }

  fun mesh::make(core &core, std::string_view fpath) -> sptr<mesh>
  {
    return make_sptr<mesh>(core, fpath);
  }

}
