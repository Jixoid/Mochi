/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/rhi.hh"
#include "mochi/rhi/vtype.hh"
#include "mochi/types.hh"
#include "mochi/rhi/shader.hh"
#include <vulkan/vulkan.hpp>
#include <vulkan/vulkan_raii.hpp>



namespace mochi::rhi
{
  // Enums
  enum struct VertexInputRate: u8 {
    PerVertex   = 0,
    PerInstance = 1,
  };

  enum struct DescriptorType: u32 {
    UniformBuffer         = 6,
    UniformBufferDynamic  = 8,
    StorageBuffer         = 7,
    StorageBufferDynamic  = 9,
    TextureSampler        = 1,
    StorageImage          = 3,
    SeparateImage         = 2,
    SeparateSampler       = 0,
    AccelerationStructure = 1000150000,
  };

  enum struct PipelineKind: u32 {
    Graphic    = 0,
    Compute    = 1,
    RayTracing = 1000165000,
  };

  enum struct PolygonMode: u8 {
    Fill  = 0,
    Line  = 1,
    Point = 2,
  };

  enum struct PrimitiveTopology: u8 {
    PointList     = 0,
    LineList      = 1,
    LineStrip     = 2,
    TriangleList  = 3,
    TriangleStrip = 4,
    TriangleFan   = 5,
    PatchList     = 10
  };



  // Helpers
  struct PushConstantSlot {
    public:
      explicit PushConstantSlot(ShaderStageFlags stage, std::vector<vt> types)
        : m_stage(stage)
        , m_types(types)
      {}

    private:
      ShaderStageFlags m_stage;
      std::vector<vt> m_types;

    public:
      fun  stage() const { return m_stage; }
      fun& types() const { return m_types; }
  };
  using PushConstantList = std::vector<PushConstantSlot>;


  struct VertexBindSlot {
    public:
      explicit VertexBindSlot(u64 stride, std::vector<vt> items, VertexInputRate inputRate, u8 index)
        : m_stride(stride)
        , m_items(items)
        , m_inputRate(inputRate)
        , m_index(index)
      {}

    private:
      u64 m_stride;
      std::vector<vt> m_items;
      VertexInputRate m_inputRate;
      u8 m_index;

    public:
      fun& items() const { return m_items; }
      fun  stride() const { return m_stride; }
      fun  inputRate() const { return m_inputRate; }
      fun  index() const { return m_index; }
  };
  using VertexBindList = std::vector<VertexBindSlot>;


  struct DescriptorSlotOne {
    public:
      explicit DescriptorSlotOne(DescriptorType kind, ShaderStageFlags stage)
        : m_kind(kind)
        , m_stage(stage)
      {}

    private:
      DescriptorType m_kind;
      ShaderStageFlags m_stage;

    public:
      fun kind() const { return m_kind; }
      fun stage() const { return m_stage; }
  };
  using DescriptorSlot = std::vector<DescriptorSlotOne>;
  using DescriptorList = std::vector<DescriptorSlot>;



  // External
  struct PipelineMeta;
  struct Pipeline;

  extern "C" fun MochiRHI_MakePipelineMeta(PushConstantList push, VertexBindList vert, DescriptorList desc) -> PipelineMeta*;
  extern "C" fun MochiRHI_MakePipeline(
    rhi::DeviceManager &dmng, PipelineMeta *info, std::vector<sptr<Shader>> shaders,
    PolygonMode polymode, PrimitiveTopology primitiveTopology,
    Format color_format, Format depth_format
  ) -> Pipeline*;



  // Interface
  struct PipelineMeta: noncopy {
    protected:
      PipelineMeta() = default;

    public:
      virtual ~PipelineMeta() = default;

      static fun make(PushConstantList push, VertexBindList vert, DescriptorList desc) {
        return make_sptr(MochiRHI_MakePipelineMeta(std::move(push), std::move(vert), std::move(desc)));
      }

    protected:
      PushConstantList m_push;
      VertexBindList m_vert;
      DescriptorList m_desc;

    public:
      fun& push() const { return m_push; }
      fun& vert() const { return m_vert; }
      fun& desc() const { return m_desc; }
  };


  struct Pipeline: noncopy {
    protected:
      Pipeline() = default;
      
    public:
      virtual ~Pipeline() = default;

      static fun make(
        rhi::DeviceManager &dmng, PipelineMeta *meta, std::vector<sptr<Shader>> shaders,
        PolygonMode polymode, PrimitiveTopology primitiveTopology,
        Format color_format, Format depth_format
      ) {
        return make_sptr(MochiRHI_MakePipeline(dmng, meta, std::move(shaders), polymode, primitiveTopology, color_format, depth_format));
      }
    

    protected:
      PipelineMeta *m_meta;
      PipelineKind m_kind;
      
    public:
      fun meta() { return m_meta; }
      fun kind() const { return m_kind; }
  };
  
}
