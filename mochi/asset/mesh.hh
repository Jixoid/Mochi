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
#include <span>



namespace mochi::asset
{

  struct vertex_t {
    vec3<f32> position;
    vec3<f32> normal;
    vec3<f32> color;
    vec2<f32> uv;
  };
  
  
  
  struct Mesh {
    public:
      explicit Mesh(sptr<rhi::Buffer> data, std::vector<offs> offs, std::vector<sptr<asset::Material>> material, std::vector<int> map);
      
      explicit Mesh(Core &core, std::span<char> file, std::string_view ext);
      
    public:
      static fun make(sptr<rhi::Buffer> data, std::vector<offs> offs, std::vector<sptr<asset::Material>> material, std::vector<int> map) {
        return make_sptr(new Mesh(data, offs, material, map));
      }
      
      static fun make(Core &core, std::span<char> file, std::string_view ext) {
        return make_sptr(new Mesh(core, file, ext));
      }
      

    private:
      sptr<rhi::Buffer> m_data;
      std::vector<offs> m_offs;
      std::vector<sptr<asset::Material>> m_material;
      std::vector<int>  m_map;

    public:
      fun& data() { return m_data; }
      fun& offs() { return m_offs; }
      fun& material() { return m_material; }
      fun& map() { return m_map; }
  };

}
