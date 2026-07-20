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
#include "mochi/rhi/manager/allocator.hh"
#include "mochi/rhi/manager/uploader.hh"
#include "mochi/core/engine.hh"
#include "mochi/except.hh"

#include "stb_image.h"



namespace mochi::asset
{

  Texture2::Texture2(Engine &eng, sptr<rhi::Image2> data): m_data(data) {
    auto& device = eng.sub<rhi::Device>();
    auto& alloc = eng.sub<rhi::Allocator>();

    m_view = m_data->makeView();
    m_sampler = alloc.allocSampler2({rhi::SamplerAddressMode::Repeat, rhi::SamplerAddressMode::Repeat, rhi::SamplerAddressMode::Repeat}, rhi::SamplerFilter::Linear, rhi::SamplerFilter::Linear);
    m_id = device.writeTextureDescriptor(m_view, m_sampler);
  }

  Texture2::Texture2(Engine &eng, const std::string &fpath) {
    int texWidth, texHeight, texChannels;
    stbi_uc* pixels = stbi_load(fpath.c_str(), &texWidth, &texHeight, &texChannels, STBI_rgb_alpha);
    if (!pixels) throw mochi::asset_error("Failed to load texture!");

    auto& alloc = eng.sub<rhi::Allocator>();
    auto& upload = eng.sub<rhi::Uploader>();

    // Initialize texture and transfer to vram
    m_data = alloc.allocImage2(
      {static_cast<u32>(texWidth), static_cast<u32>(texHeight)}, 
      rhi::Format::v4norm8U,
      flags(rhi::ImageUsage::Sampled) | rhi::ImageUsage::TransferDst, 
      rhi::ImageTiling::Optimal, 
      {},
      rhi::AllocationLocation::PreferDevice
    );

    upload.copyMemoryToImage(rhi::TransferTime::Now, pixels, m_data.get());
    stbi_image_free(pixels);

    auto& dev_mgr = eng.sub<rhi::Device>();
    m_view = m_data->makeView();
    m_sampler = alloc.allocSampler2({rhi::SamplerAddressMode::Repeat, rhi::SamplerAddressMode::Repeat, rhi::SamplerAddressMode::Repeat}, rhi::SamplerFilter::Linear, rhi::SamplerFilter::Linear);
    m_id = dev_mgr.writeTextureDescriptor(m_view, m_sampler);
  }

  Texture2::Texture2(Engine &eng, u32 width, u32 height, const void *pixels) {
    auto& device = eng.sub<rhi::Device>();
    auto& alloc = eng.sub<rhi::Allocator>();
    auto& upload = eng.sub<rhi::Uploader>();

    m_data = alloc.allocImage2(
      {width, height}, 
      rhi::Format::v4norm8U,
      flags(rhi::ImageUsage::Sampled) | rhi::ImageUsage::TransferDst, 
      rhi::ImageTiling::Optimal, 
      {},
      rhi::AllocationLocation::PreferDevice
    );
    upload.copyMemoryToImage(rhi::TransferTime::Now, const_cast<void*>(pixels), m_data.get());


    m_view = m_data->makeView();
    m_sampler = alloc.allocSampler2({rhi::SamplerAddressMode::Repeat, rhi::SamplerAddressMode::Repeat, rhi::SamplerAddressMode::Repeat}, rhi::SamplerFilter::Linear, rhi::SamplerFilter::Linear);
    m_id = device.writeTextureDescriptor(m_view, m_sampler);
  }

}
