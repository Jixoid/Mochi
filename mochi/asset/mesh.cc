/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/asset/material.hh"
#include "mochi/asset/texture.hh"
#include "mochi/basis.hh"
#include "mochi/asset/mesh.hh"
#include "mochi/except.hh"
#include "mochi/module/memory.hh"
#include "mochi/reader/reader.hh"
#include "mochi/rhi/image.hh"
#include "mochi/rhi/vtype.hh"
#include "mochi/types.hh"
#include "mochi/core.hh"
#include <cstring>
#include <string>
#include <string_view>
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>
#include "stb_image.h"



namespace mochi::asset
{
  
  auto vertex_i = rhi::info<rhi::buffer>::make(
    sizeof(vertex_t),
    rhi::vt::make_list<
      vec3<f32>, // Position
      vec3<f32>, // Normal
      vec3<f32>, // Color
      vec2<f32>  // UV
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

  inline fun include_images(core &core, gltf_image &raw_image) -> sptr<rhi::image2>
  {
    int texWidth, texHeight, texChannels;
    stbi_uc* pixels = stbi_load_from_memory(raw_image.data.data(), raw_image.data.size(), &texWidth, &texHeight, &texChannels, STBI_rgb_alpha);
    if (!pixels) throw mochi::asset_error("Failed to load texture!");

    
    auto ret = rhi::image2::make(
      core.sub<module::device>(), 
      core.sub<module::memory>(), 
      core.sub<module::renderer>().cmd_pool(),
      texWidth, texHeight, pixels
    );

    stbi_image_free(pixels);

    return ret;
  }



  mesh::mesh(sptr<rhi::buffer> data, std::vector<::offs> offs, std::vector<sptr<asset::material>> material)
    : m_data(data)
    , m_offs(offs)
    , m_material(material)
  {}

  mesh::mesh(core &core, std::string_view fpath)
  {
    mappedFile mfile((std::string)fpath);
    
    std::vector<vertex_t> final_data;
    std::vector<::offs>   final_offs;
    std::vector<sptr<asset::material>> final_material;
    std::vector<int>      final_map;


    if (fpath.ends_with(".obj")) {
      auto raw = read<ft_wavefront>(mfile);

      auto asset = make_sptr<asset::material>(core);
      asset->setColor({0,0,0});

      final_data = std::move(build_vertices(raw));
      final_offs = {{0, final_data.size()}};
      final_material = {asset};
      final_map = {0};
    } 
    ef (fpath.ends_with(".glb") || fpath.ends_with(".gltf")) {
      auto raw = read<ft_gltf>(mfile);

      final_data = std::move(raw.vertices);
      final_offs = std::move(raw.offsets);

      for (auto &X: raw.images) {
        if (X.data.empty()) {
          auto mat = make_sptr<asset::material>(core);
          mat->setColor({0.5f, 0.5f, 0.5f}); // Gri renk
          final_material.push_back(mat);
        }
        else {
          auto img = include_images(core, X);
          auto tex = make_sptr<asset::texture2>(core, img);

          auto mat = make_sptr<asset::material>(core);
          mat->setTexture(tex);
          
          final_material.push_back(mat);
        }
      }

      final_map = std::move(raw.image_map);
    }


    m_data = core.sub<module::memory>().load_VertexBuffer(vertex_i, final_data.size(),
      [&final_data](void* _data) {
        memcpy(_data, final_data.data(), vertex_i->stride() * final_data.size());
      }
    );
    m_offs = std::move(final_offs);
    m_map = std::move(final_map);
    m_material = std::move(final_material);
  }

}
