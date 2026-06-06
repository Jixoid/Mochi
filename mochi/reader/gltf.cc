/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/asset/mesh.hh"
#include "mochi/reader/reader.hh"
#include "mochi/except.hh"
#define CGLTF_IMPLEMENTATION
#include "cgltf.h"



namespace mochi
{

  template<>
  fun read<ft_gltf>(data src) -> gltf_obj
  {
    gltf_obj result;
    
    cgltf_options options = {};
    cgltf_data* data = NULL;
    
    

    if (cgltf_parse(&options, src.ptr(), src.size(), &data) != cgltf_result_success)
      throw mochi::asset_error("glTF file cannot parsed.");

    
    if (cgltf_load_buffers(&options, data, nil) != cgltf_result_success) {
      cgltf_free(data);
      throw mochi::asset_error("glTF buffers cannot loaded.");
    }


    // Texture
    for (cgltf_size i = 0; i < data->images_count; i++)
    {
      cgltf_image *img = &data->images[i];
      gltf_image extracted_img;

      if (img->buffer_view && img->buffer_view->buffer->data)
      {
        u8* img_data_ptr = static_cast<u8*>(img->buffer_view->buffer->data) + img->buffer_view->offset;
        u0 img_size = img->buffer_view->size;

        if (img->name) extracted_img.name = img->name;
        if (img->mime_type) extracted_img.mime_type = img->mime_type;
        
        extracted_img.data.assign(img_data_ptr, img_data_ptr + img_size);
      }

      result.images.push_back(std::move(extracted_img));
    }
    

    // Vertex
    for (cgltf_size i = 0; i < data->meshes_count; i++)
    {
      cgltf_mesh *mesh = &data->meshes[i];
      
      for (cgltf_size j = 0; j < mesh->primitives_count; j++)
      {
        cgltf_primitive *prim = &mesh->primitives[j];
        
        cgltf_size vertex_count = prim->attributes[0].data->count;
        std::vector<asset::vertex_t> temp_vertices(vertex_count);


        for (auto &v: temp_vertices) v.color = {1,1,1};

        // Extract attributes
        for (cgltf_size a = 0; a < prim->attributes_count; a++) {
          cgltf_attribute *attr = &prim->attributes[a];
          
          if (attr->type == cgltf_attribute_type_position) {
            for (cgltf_size v = 0; v < vertex_count; v++)
              cgltf_accessor_read_float(attr->data, v, &temp_vertices[v].position.x, 3);
          } 
          ef (attr->type == cgltf_attribute_type_normal) {
            for (cgltf_size v = 0; v < vertex_count; v++)
              cgltf_accessor_read_float(attr->data, v, &temp_vertices[v].normal.x, 3);
          } 
          ef (attr->type == cgltf_attribute_type_texcoord) {
            for (cgltf_size v = 0; v < vertex_count; v++)
              cgltf_accessor_read_float(attr->data, v, &temp_vertices[v].uv.x, 2);
          }
        }

        // Unroll
        auto legsize = result.vertices.size();

        if (prim->indices) {
          for (cgltf_size id = 0; id < prim->indices->count; id++) {
            cgltf_size index = cgltf_accessor_read_index(prim->indices, id);
            result.vertices.push_back(temp_vertices[index]);
          }
        } else {
          result.vertices.insert(result.vertices.end(), temp_vertices.begin(), temp_vertices.end());
        }

        result.offsets.push_back({legsize, result.vertices.size() - legsize});
        int img_idx = -1;
        if (prim->material && prim->material->has_pbr_metallic_roughness) {
          cgltf_texture_view* tex_view = &prim->material->pbr_metallic_roughness.base_color_texture;
          if (tex_view->texture && tex_view->texture->image) {
            img_idx = tex_view->texture->image - data->images;
          }
        }
        result.image_map.push_back(img_idx);
      }
    }

    
    cgltf_free(data);
    return result;
  }
  
}
