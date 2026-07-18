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
#include "mochi/core/core.hh"
#include "mochi/except.hh"

#include "stb_image.h"



namespace mochi::asset
{

  Texture2::Texture2(Core &core, sptr<rhi::Image2> data): m_data(data) {
    auto& alloc_mgr = core.sub<rhi::mng::AllocManager>();
    auto& dev_mgr = core.sub<rhi::mng::DeviceManager>();
    m_view = m_data->makeView();
    m_sampler = alloc_mgr.allocSampler2({rhi::SamplerAddressMode::Repeat, rhi::SamplerAddressMode::Repeat, rhi::SamplerAddressMode::Repeat}, rhi::SamplerFilter::Linear, rhi::SamplerFilter::Linear);
    m_id = dev_mgr.writeTextureDescriptor(m_view, m_sampler);
  }

  Texture2::Texture2(Core &core, const std::string &fpath) {
    int texWidth, texHeight, texChannels;
    stbi_uc* pixels = stbi_load(fpath.c_str(), &texWidth, &texHeight, &texChannels, STBI_rgb_alpha);
    if (!pixels) throw mochi::asset_error("Failed to load texture!");

    auto& alloc_mgr = core.sub<rhi::mng::AllocManager>();
    auto& transfer_mgr = core.sub<rhi::mng::TransferManager>();

    // Initialize texture and transfer to vram
    m_data = alloc_mgr.allocImage2(
      {static_cast<u32>(texWidth), static_cast<u32>(texHeight)}, 
      rhi::Format::v4norm8U,
      flags(rhi::ImageUsage::Sampled) | rhi::ImageUsage::TransferDst, 
      rhi::ImageTiling::Optimal, 
      {},
      rhi::mng::AllocationLocation::PreferDevice
    );

    transfer_mgr.copyMemoryToImage(rhi::mng::TransferTime::Now, pixels, m_data.get());
    stbi_image_free(pixels);

    auto& dev_mgr = core.sub<rhi::mng::DeviceManager>();
    m_view = m_data->makeView();
    m_sampler = alloc_mgr.allocSampler2({rhi::SamplerAddressMode::Repeat, rhi::SamplerAddressMode::Repeat, rhi::SamplerAddressMode::Repeat}, rhi::SamplerFilter::Linear, rhi::SamplerFilter::Linear);
    m_id = dev_mgr.writeTextureDescriptor(m_view, m_sampler);
  }

  Texture2::Texture2(Core &core, u32 width, u32 height, const void *pixels) {
    auto& alloc_mgr = core.sub<rhi::mng::AllocManager>();
    auto& transfer_mgr = core.sub<rhi::mng::TransferManager>();

    m_data = alloc_mgr.allocImage2(
      {width, height}, 
      rhi::Format::v4norm8U,
      flags(rhi::ImageUsage::Sampled) | rhi::ImageUsage::TransferDst, 
      rhi::ImageTiling::Optimal, 
      {},
      rhi::mng::AllocationLocation::PreferDevice
    );
    transfer_mgr.copyMemoryToImage(rhi::mng::TransferTime::Now, const_cast<void*>(pixels), m_data.get());

    auto& dev_mgr = core.sub<rhi::mng::DeviceManager>();
    m_view = m_data->makeView();
    m_sampler = alloc_mgr.allocSampler2({rhi::SamplerAddressMode::Repeat, rhi::SamplerAddressMode::Repeat, rhi::SamplerAddressMode::Repeat}, rhi::SamplerFilter::Linear, rhi::SamplerFilter::Linear);
    m_id = dev_mgr.writeTextureDescriptor(m_view, m_sampler);
  }



    
  fun Texture2::make(Core &core, sptr<rhi::Image2> data) -> sptr<Texture2> {
    return make_sptr<Texture2>(core, data);
  }

  fun Texture2::make(Core &core, const std::string &fpath) -> sptr<Texture2> {
    return make_sptr<Texture2>(core, fpath);
  }

  fun Texture2::make(Core &core, u32 width, u32 height, const void *pixels) -> sptr<Texture2> {
    return make_sptr<Texture2>(core, width, height, pixels);
  }

}
