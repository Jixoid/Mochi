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
      explicit Texture2(Engine &eng, sptr<rhi::Image2> data);

      explicit Texture2(Engine &eng, const std::string &fpath);

      explicit Texture2(Engine &eng, u32 width, u32 height, const void *pixels);

    public:
      static fun make(Engine &eng, sptr<rhi::Image2> data) {
        return make_sptr<Texture2>(eng, data);
      }

      static fun make(Engine &eng, const std::string &fpath) {
        return make_sptr<Texture2>(eng, fpath);
      }

      static fun make(Engine &eng, u32 width, u32 height, const void *pixels) {
        return make_sptr<Texture2>(eng, width, height, pixels);
      }


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
