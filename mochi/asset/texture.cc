/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/asset/texture.hh"
#include "mochi/rhi/image.hh"
#include "mochi/rhi/manager/alloc_manager.hh"
#include "mochi/rhi/manager/transfer_manager.hh"
#include "mochi/core.hh"
#include "mochi/except.hh"

#include "stb_image.h"



namespace mochi::asset
{

  texture2::texture2(core &core, sptr<rhi::Image2> data)
    : m_data(data)
  {}

  texture2::texture2(core &core, const std::string &fpath)
  {
    int texWidth, texHeight, texChannels;
    stbi_uc* pixels = stbi_load(fpath.c_str(), &texWidth, &texHeight, &texChannels, STBI_rgb_alpha);
    if (!pixels) throw mochi::asset_error("Failed to load texture!");

    auto& alloc_mgr = core.sub<rhi::AllocManager>();
    auto& transfer_mgr = core.sub<rhi::TransferManager>();

    // Initialize texture and transfer to vram
    m_data = alloc_mgr.allocImage2(
      {static_cast<u32>(texWidth), static_cast<u32>(texHeight)}, 
      rhi::Format::v4norm8U,
      flags(rhi::ImageUsage::Sampled) | rhi::ImageUsage::TransferDst, 
      rhi::ImageTiling::Optimal, 
      rhi::AllocationCreateFlags(), 
      rhi::AllocationLocation::PreferDevice
    );

    transfer_mgr.copyMemoryToImage(rhi::TransferTime::Now, pixels, m_data.get());

    stbi_image_free(pixels);
  }

  texture2::texture2(core &core, u32 width, u32 height, const void *pixels) {
    auto& alloc_mgr = core.sub<rhi::AllocManager>();
    auto& transfer_mgr = core.sub<rhi::TransferManager>();

    m_data = alloc_mgr.allocImage2(
      {width, height}, 
      rhi::Format::v4norm8U,
      flags(rhi::ImageUsage::Sampled) | rhi::ImageUsage::TransferDst, 
      rhi::ImageTiling::Optimal, 
      rhi::AllocationCreateFlags(), 
      rhi::AllocationLocation::PreferDevice
    );
    transfer_mgr.copyMemoryToImage(rhi::TransferTime::Now, const_cast<void*>(pixels), m_data.get());
  }



    
  fun texture2::make(core &core, sptr<rhi::Image2> data) -> sptr<texture2> {
    return make_sptr<texture2>(core, data);
  }

  fun texture2::make(core &core, const std::string &fpath) -> sptr<texture2> {
    return make_sptr<texture2>(core, fpath);
  }

  fun texture2::make(core &core, u32 width, u32 height, const void *pixels) -> sptr<texture2> {
    return make_sptr<texture2>(core, width, height, pixels);
  }

}
