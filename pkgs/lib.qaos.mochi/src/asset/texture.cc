/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "basis.hh"
#include "mochi/asset/texture.hh"
#include "mochi/rhi/image.hh"
#include "mochi/module/renderer.hh"
#include "mochi/module/memory.hh"
#include "mochi/core.hh"
#include "mochi/except.hh"
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>
#include "stb_image.h"



namespace mochi::asset
{

  texture2::texture2(core &core, sptr<rhi::image2> data)
    : m_data(data)
  {}

  texture2::texture2(core &core, const std::string &fpath)
  {
    int texWidth, texHeight, texChannels;
    stbi_uc* pixels = stbi_load(fpath.c_str(), &texWidth, &texHeight, &texChannels, STBI_rgb_alpha);
    if (!pixels) throw mochi::asset_error("Failed to load texture!");

    // Initialize texture and transfer to vram
    m_data = make_sptr<rhi::image2>(
      core.sub<module::device>(), 
      core.sub<module::memory>(), 
      core.sub<module::renderer>().cmd_pool(),
      texWidth, texHeight, pixels
    );

    stbi_image_free(pixels);
  }



    
  fun texture2::make(core &core, sptr<rhi::image2> data) -> sptr<texture2>
  {
    return make_sptr<texture2>(core, data);
  }

  fun texture2::make(core &core, const std::string &fpath) -> sptr<texture2>
  {
    return make_sptr<texture2>(core, fpath);
  }

}
