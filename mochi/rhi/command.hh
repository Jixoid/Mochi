/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/math/extent.hh"
#include "mochi/rhi/pipeline.hh"
#include "mochi/rhi/render_target.hh"
#include "mochi/rhi/buffer.hh"



namespace mochi::rhi
{
  // Helpers
  struct Viewport{
    public:
      Viewport(extent<3,f32> min, extent<3,f32> max): m_min(min), m_max(max) {}
      Viewport(f32 minX, f32 maxX, f32 minY, f32 maxY, f32 minZ, f32 maxZ): m_min(minX,minY,minZ), m_max(maxX, maxY, maxZ) {}

    private:
      extent<3,f32> m_min{}, m_max{};

    public:
      fun& min() const { return m_min; }
      fun& max() const { return m_max; }
  };

  struct Rect2D {
    public:
      Rect2D(extent<2,i32> offset, extent<2,u32> extent): m_offset(offset), m_extent(extent) {}
      Rect2D(i32 x, i32 y, u32 w, u32 h): m_offset(x,y), m_extent(w,h) {}

    private:
      extent<2,i32> m_offset{};
      extent<2,u32> m_extent{};

    public:
      fun& offset() const { return m_offset; }
      fun& extent() const { return m_extent; }
  };


  // Interface
  struct Command: noncopy {
    protected:
      explicit Command() = default;

    public:
      virtual ~Command() = default;

    
    public:
      virtual fun begin() -> void = 0;
      virtual fun end() -> void = 0;
      virtual fun beginRendering(const RenderTarget &target, const std::array<float, 4> &clear_color) -> void = 0;
      virtual fun endRendering(const RenderTarget &target) -> void = 0;

      virtual fun bindPipeline(Pipeline *pipe) -> void = 0;
      virtual fun bindDescriptorHeap(sptr<Buffer> resource_heap, sptr<Buffer> sampler_heap, u64 resource_size, u64 sampler_size) -> void = 0;
      virtual fun pushConstant(ShaderStageFlags stage, u32 off, data dat) -> void = 0;

      virtual fun draw(offs vertex, offs inst) -> void = 0;
      virtual fun dispatch(extent<3,u32> ext) -> void = 0;

      virtual fun setViewport(u32 first, std::vector<Viewport> viewports) -> void = 0;
      virtual fun setScissor(u32 first, std::vector<Rect2D> scissors) -> void = 0;
  };

}
