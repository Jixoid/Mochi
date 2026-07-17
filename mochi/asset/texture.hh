/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/image.hh"
#include "mochi/rhi/sampler.hh"



namespace mochi::asset
{

  struct Texture2 {
    public:
      explicit Texture2(Core &core, sptr<rhi::Image2> data);

      explicit Texture2(Core &core, const std::string &fpath);

      explicit Texture2(Core &core, u32 width, u32 height, const void *pixels);

    public:
      static fun make(Core &core, sptr<rhi::Image2> data) -> sptr<Texture2>;

      static fun make(Core &core, const std::string &fpath) -> sptr<Texture2>;

      static fun make(Core &core, u32 width, u32 height, const void *pixels) -> sptr<Texture2>;


    private:
      sptr<rhi::Image2> m_data;
      sptr<rhi::ImageView2> m_view;
      sptr<rhi::Sampler2> m_sampler;
      u32 m_id{0}; // ID in the global descriptor heap

    public:
      fun data() const -> sptr<rhi::Image2> { return m_data; }
      fun view() const -> sptr<rhi::ImageView2> { return m_view; }
      fun sampler() const -> sptr<rhi::Sampler2> { return m_sampler; }
      
      fun id() const -> u32 { return m_id; }
      
      fun set_id(u32 id) { m_id = id; }
  };

}
