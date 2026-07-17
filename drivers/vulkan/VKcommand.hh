/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "drivers/vulkan/VKpipeline.hh"
#include "mochi/basis.hh"
#include "mochi/rhi/command.hh"
#include "mochi/rhi/pipeline.hh"
#include "mochi/rhi/render_target.hh"
#include <vulkan/vulkan_raii.hpp>



namespace mochi::rhi::vulkan
{

  struct VK_Command final: public rhi::Command {
    
    public:
      explicit VK_Command(vk::raii::CommandBuffer cmd) : m_cmd(std::move(cmd)) {}
      
    private:
      vk::raii::CommandBuffer m_cmd;
      VK_Pipeline *m_pipe{};
    
    public:
      fun begin() -> void override;
      fun end() -> void override;
      fun beginRendering(const RenderTarget &target, const std::array<float, 4> &clear_color) -> void override;
      fun endRendering(const RenderTarget &target) -> void override;

      fun bindPipeline(rhi::Pipeline *pipe) -> void override;
      fun bindDescriptorHeap(sptr<rhi::Buffer> resource_heap, sptr<rhi::Buffer> sampler_heap, u64 resource_size, u64 sampler_size) -> void override;
      fun pushConstant(rhi::ShaderStageFlags stage, u32 off, data dat) -> void override;

      fun draw(offs vertex, offs inst) -> void override;
      fun dispatch(extent<3,u32> ext) -> void override;

      fun setViewport(u32 first, std::vector<Viewport> viewports) -> void override;
      fun setScissor(u32 first, std::vector<Rect2D> scissors) -> void override;

    public:
      fun& get() { return m_cmd; }
  };

}
