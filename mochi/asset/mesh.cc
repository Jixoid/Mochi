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
#include "mochi/reader/reader.hh"
#include "mochi/rhi/image.hh"
#include "mochi/rhi/manager/alloc_manager.hh"
#include "mochi/rhi/manager/transfer_manager.hh"
#include "mochi/rhi/rhi.hh"
#include "mochi/rhi/vtype.hh"
#include "mochi/types.hh"
#include "mochi/core/core.hh"
#include <string>
#include <string_view>

#include "stb_image.h"



namespace mochi::asset
{
  
  inline fun build_vertices(wf_obj &raw_data) -> std::vector<vertex_t> {
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

  inline fun include_images(Core &core, gltf_image &raw_image) -> sptr<rhi::Image2> {
    int texWidth, texHeight, texChannels;
    stbi_uc* pixels = stbi_load_from_memory(raw_image.data.data(), raw_image.data.size(), &texWidth, &texHeight, &texChannels, STBI_rgb_alpha);
    if (!pixels) throw mochi::asset_error("Failed to load texture!");

    auto& alloc_mgr = core.sub<rhi::AllocManager>();
    auto& transfer_mgr = core.sub<rhi::TransferManager>();

    auto img = alloc_mgr.allocImage2(
      { (u32)texWidth, (u32)texHeight },
      rhi::Format::v4norm8U,
      flags(rhi::ImageUsage::TransferDst) | rhi::ImageUsage::Sampled,
      rhi::ImageTiling::Optimal,
      rhi::AllocationCreateFlags(),
      rhi::AllocationLocation::PreferDevice
    );
    
    transfer_mgr.copyMemoryToImage(rhi::TransferTime::Now, pixels, img.get());

    stbi_image_free(pixels);

    return img;
  }



  Mesh::Mesh(sptr<rhi::Buffer> data, std::vector<::offs> offs, std::vector<sptr<asset::Material>> material, std::vector<int> map)
    : m_data(data)
    , m_offs(offs)
    , m_material(material)
    , m_map(map)
  {}

  Mesh::Mesh(Core &core, std::span<char> file, std::string_view ext) {
    ::data mfile{file.data(), file.size_bytes()};
    
    std::vector<vertex_t> final_data;
    std::vector<::offs>   final_offs;
    std::vector<sptr<asset::Material>> final_material;
    std::vector<int>      final_map;


    if (ext == ".obj") {
      auto raw = read<ft_wavefront>(mfile);

      auto asset = make_sptr<asset::Material>(core);
      asset->setColor({0,0,0});

      final_data = std::move(build_vertices(raw));
      final_offs = {{0, final_data.size()}};
      final_material = {asset};
      final_map = {0};
    } 
    ef (ext == ".glb" || ext == ".gltf") {
      auto raw = read<ft_gltf>(mfile);

      final_data = std::move(raw.vertices);
      final_offs = std::move(raw.offsets);

      for (auto &X: raw.images) {
        if (X.data.empty()) {
          auto mat = make_sptr<asset::Material>(core);
          mat->setColor({0.5f, 0.5f, 0.5f});
          final_material.push_back(mat);
        }
        else {
          auto img = include_images(core, X);
          auto tex = make_sptr<asset::Texture2>(core, img);

          auto mat = make_sptr<asset::Material>(core);
          mat->setTexture(tex);
          
          final_material.push_back(mat);
        }
      }

      auto default_mat = make_sptr<asset::Material>(core);
      default_mat->setColor({0.8f, 0.8f, 0.8f});
      final_material.push_back(default_mat);
      int default_idx = final_material.size() - 1;

      final_map.reserve(raw.image_map.size());
      for (int idx : raw.image_map) {
        if (idx == -1) {
          final_map.push_back(default_idx);
        }
        else {
          final_map.push_back(idx);
        }
      }
    }


    auto& alloc_mgr = core.sub<rhi::AllocManager>();
    auto& transfer_mgr = core.sub<rhi::TransferManager>();

    m_data = alloc_mgr.allocBuffer(
      sizeof(vertex_t) * final_data.size(),
      flags(rhi::BufferUsage::DeviceAddress) | rhi::BufferUsage::TransferDst,
      rhi::AllocationCreate::Mapped,
      rhi::AllocationLocation::PreferDevice
    );

    transfer_mgr.copyMemoryToBuffer(rhi::TransferTime::Now, final_data.data(), m_data.get());
    m_offs = std::move(final_offs);
    m_map = std::move(final_map);
    m_material = std::move(final_material);
  }

}
