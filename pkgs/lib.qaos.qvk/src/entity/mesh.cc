/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "Basis.hh"
#include "qvk/entity/mesh.hh"
#include "qvk/module/memory.hh"
#include "qvk/reader/reader.hh"
#include "qvk/types.hh"
#include "qvk/core.hh"
#include <cstring>
#include <string>
#include <string_view>
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace qvk
{
  
  qvk::info<qvk::buffer> vertex_i = qvk::info<qvk::buffer>(
    sizeof(vertex_t),
    gt::make_list<
      vec3<f32>, // position
      vec3<f32>, // normal
      vec3<f32>, // color
      vec2<f32>  // uv
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

          // 1. Pozisyon
          if (tri_indices[j].v_idx >= 0 && tri_indices[j].v_idx < raw_data.v.size())
            v.position = raw_data.v[tri_indices[j].v_idx];

          // 2. UV
          if (tri_indices[j].vt_idx >= 0 && tri_indices[j].vt_idx < raw_data.vt.size())
            v.uv = raw_data.vt[tri_indices[j].vt_idx];
          else
            v.uv = {0.0f, 0.0f};

          // 3. Normal
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



  mesh::mesh(qvk::buffer *data)
    : m_data(data)
  {}

  mesh::mesh(core &core, std::string_view fpath)
  {
    mappedFile mfile((std::string)fpath);
    
    auto raw = read<ft_wavefront>(mfile);
    
    auto vtx = build_vertices(raw);
    
    m_data = core.sub<memory>().load_VertexBuffer(&vertex_i, vtx.size(),
      [&vtx](void* _data)
      {
        memcpy(_data, vtx.data(), vertex_i.stride()*vtx.size());
      }
    );
  }


  fun mesh::make(core &core, qvk::buffer *data) -> mesh*
  {
    auto obj = new mesh(data);

    core.sub<memory>().push(obj);
    return obj;
  }

  fun mesh::make(core &core, std::string_view fpath) -> mesh*
  {
    auto obj = new mesh(core, fpath);

    core.sub<memory>().push(obj);
    return obj;
  }

}
