/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"



namespace mochi::rhi
{

  enum struct ShaderStage: u32
  {
    Vertex       = 0x00000001,
    Pixel        = 0x00000010,
    Compute      = 0x00000020,
    Task         = 0x00000040,
    Mesh         = 0x00000080,
    RayGen       = 0x00000100,
    AnyHit       = 0x00000200,
    ClosestHit   = 0x00000400,
    Miss         = 0x00000800,
    Intersection = 0x00001000,
    Callable     = 0x00002000,
  };
  using ShaderStageFlags = flags<ShaderStage>;


  enum struct BufferUsage: u32
  {
    TransferSrc    = 0x00000001,
    TransferDst    = 0x00000002,
    UniformBuffer  = 0x00000010,
    StorageBuffer  = 0x00000020,
    IndexBuffer    = 0x00000040,
    VertexBuffer   = 0x00000080,
    IndirectBuffer = 0x00000100,
    DeviceAddress  = 0x00020000,
  };
  using BufferUsageFlags = flags<BufferUsage>;
  
  
  enum struct BufferCreate: u32
  {
    Dedicated           = 0x00000001,
    Mapped              = 0x00000004,
    HostSequentialWrite = 0x00000400,
    HostRandomAccess    = 0x00000800,
    HostTransfer        = 0x00001000,
  };
  using BufferCreateFlags = flags<BufferCreate>;


  enum struct BufferLocation: u8
  {
    Auto         = 7,
    PreferDevice = 8,
    PreferHost   = 9,
  };
  

  enum struct VertexInputRate: u8 {
    PerVertex   = 0,
    PerInstance = 1,
  };


  enum struct DescriptorType: u32
  {
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

  enum struct PolygonMode: u8
  {
    Fill  = 0,
    Line  = 1,
    Point = 2,
  };

  enum struct PrimitiveTopology: u8
  {
    ePointList     = 0,
    eLineList      = 1,
    eLineStrip     = 2,
    eTriangleList  = 3,
    eTriangleStrip = 4,
    eTriangleFan   = 5,
    ePatchList     = 10
  };


  enum struct PipelineKind: u32
  {
    Graphic    = 0,
    Compute    = 1,
    RayTracing = 1000165000,
  };


  enum struct Format: u32
  {
    /// 8-bit
    int8U    = 13, // FORMAT_R8_UINT
    int8S    = 14, // FORMAT_R8_SINT
    norm8U   = 9,  // FORMAT_R8_UNORM
    norm8S   = 10, // FORMAT_R8_SNORM

    v2int8U  = 20, // FORMAT_R8G8_UINT
    v2int8S  = 21, // FORMAT_R8G8_SINT
    v2norm8U = 16, // FORMAT_R8G8_UNORM
    v2norm8S = 17, // FORMAT_R8G8_SNORM

    v3int8U  = 27, // FORMAT_R8G8B8_UINT
    v3int8S  = 28, // FORMAT_R8G8B8_SINT
    v3norm8U = 23, // FORMAT_R8G8B8_UNORM
    v3norm8S = 24, // FORMAT_R8G8B8_SNORM

    v4int8U  = 41, // FORMAT_R8G8B8A8_UINT
    v4int8S  = 42, // FORMAT_R8G8B8A8_SINT
    v4norm8U = 37, // FORMAT_R8G8B8A8_UNORM
    v4norm8S = 38, // FORMAT_R8G8B8A8_SNORM


    /// 16-bit
    int16U    = 74, // FORMAT_R16_UINT
    int16S    = 75, // FORMAT_R16_SINT
    norm16U   = 70, // FORMAT_R16_UNORM
    norm16S   = 71, // FORMAT_R16_SNORM
    float16   = 76, // FORMAT_R16_SFLOAT

    v2int16U  = 81, // FORMAT_R16G16_UINT
    v2int16S  = 82, // FORMAT_R16G16_SINT
    v2norm16U = 77, // FORMAT_R16G16_UNORM
    v2norm16S = 78, // FORMAT_R16G16_SNORM
    v2float16 = 83, // FORMAT_R16G16_SFLOAT

    v3int16U  = 88, // FORMAT_R16G16B16_UINT
    v3int16S  = 89, // FORMAT_R16G16B16_SINT
    v3norm16U = 84, // FORMAT_R16G16B16_UNORM
    v3norm16S = 85, // FORMAT_R16G16B16_SNORM
    v3float16 = 90, // FORMAT_R16G16B16_SFLOAT

    v4int16U  = 95, // FORMAT_R16G16B16A16_UINT
    v4int16S  = 96, // FORMAT_R16G16B16A16_SINT
    v4norm16U = 91, // FORMAT_R16G16B16A16_UNORM
    v4norm16S = 92, // FORMAT_R16G16B16A16_SNORM
    v4float16 = 97, // FORMAT_R16G16B16A16_SFLOAT


    /// 32-bit
    int32U    = 98,  // FORMAT_R32_UINT
    int32S    = 99,  // FORMAT_R32_SINT
    float32   = 100, // FORMAT_R32_SFLOAT

    v2int32U  = 101, // FORMAT_R32G32_UINT
    v2int32S  = 102, // FORMAT_R32G32_SINT
    v2float32 = 103, // FORMAT_R32G32_SFLOAT

    v3int32U  = 104, // FORMAT_R32G32B32_UINT
    v3int32S  = 105, // FORMAT_R32G32B32_SINT
    v3float32 = 106, // FORMAT_R32G32B32_SFLOAT

    v4int32U  = 107, // FORMAT_R32G32B32A32_UINT
    v4int32S  = 108, // FORMAT_R32G32B32A32_SINT
    v4float32 = 109, // FORMAT_R32G32B32A32_SFLOAT


    // 64-bit
    int64U    = 110, // FORMAT_R64_UINT
    int64S    = 111, // FORMAT_R64_SINT
    float64   = 112, // FORMAT_R64_SFLOAT

    v2int64U  = 113, // FORMAT_R64G64_UINT
    v2int64S  = 114, // FORMAT_R64G64_SINT
    v2float64 = 115, // FORMAT_R64G64_SFLOAT

    v3int64U  = 116, // FORMAT_R64G64B64_UINT
    v3int64S  = 117, // FORMAT_R64G64B64_SINT
    v3float64 = 118, // FORMAT_R64G64B64_SFLOAT

    v4int64U  = 119, // FORMAT_R64G64B64A64_UINT
    v4int64S  = 120, // FORMAT_R64G64B64A64_SINT
    v4float64 = 121, // FORMAT_R64G64B64A64_SFLOAT


    // Image/Texture
    rgba8Srgb  = 43,  // FORMAT_R8G8B8A8_SRGB
    bgra8Unorm = 44,  // FORMAT_B8G8R8A8_UNORM
    bgra8Srgb  = 50,  // FORMAT_B8G8R8A8_SRGB
    

    // Depth/Stencil
    depth16           = 124, // FORMAT_D16_UNORM
    depth32f          = 126, // FORMAT_D32_SFLOAT
    depth24_stencil8  = 129, // FORMAT_D24_UNORM_S8_UINT
    depth32f_stencil8 = 130  // FORMAT_D32_SFLOAT_S8_UINT
  };

}
