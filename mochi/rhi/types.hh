/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/math/math.hh"

#define ef else if



namespace mochi::rhi
{

  enum struct Format: u32 {
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


  struct vt {
    private:
      vt(u64 size, u64 align, Format format, u8 count = 1)
        : m_size(size)
        , m_align(align)
        , m_format(format)
        , m_count(count)
      {}


    public:
      /// @brief Create a vt instance mapped to the specified type T.
      template <typename T>
      static fun make() -> vt {
        // primitive
        if constexpr (std::is_same_v<T, u8>)  return {1, 1, Format::int8U};
        ef constexpr (std::is_same_v<T, u16>) return {2, 2, Format::int16U};
        ef constexpr (std::is_same_v<T, u32>) return {4, 4, Format::int32U};
        ef constexpr (std::is_same_v<T, u64>) return {8, 8, Format::int64U};

        ef constexpr (std::is_same_v<T, i8>)  return {1, 1, Format::int8S};
        ef constexpr (std::is_same_v<T, i16>) return {2, 2, Format::int16S};
        ef constexpr (std::is_same_v<T, i32>) return {4, 4, Format::int32S};
        ef constexpr (std::is_same_v<T, i64>) return {8, 8, Format::int64S};

        ef constexpr (std::is_same_v<T, nu8>)  return {1, 1, Format::norm8U};
        ef constexpr (std::is_same_v<T, nu16>) return {2, 2, Format::norm16U};

        ef constexpr (std::is_same_v<T, ni8>)  return {1, 1, Format::norm8S};
        ef constexpr (std::is_same_v<T, ni16>) return {2, 2, Format::norm16S};

        ef constexpr (std::is_same_v<T, f16>) return {2, 2, Format::float16};
        ef constexpr (std::is_same_v<T, f32>) return {4, 4, Format::float32};
        ef constexpr (std::is_same_v<T, f64>) return {8, 8, Format::float64};


        // vec2
        ef constexpr (std::is_same_v<T, vec2<u8>>)  return {2,   2,  Format::v2int8U};
        ef constexpr (std::is_same_v<T, vec2<u16>>) return {4,   4,  Format::v2int16U};
        ef constexpr (std::is_same_v<T, vec2<u32>>) return {8,   8,  Format::v2int32U};
        ef constexpr (std::is_same_v<T, vec2<u64>>) return {16,  16, Format::v2int64U};

        ef constexpr (std::is_same_v<T, vec2<i8>>)  return {2,   2,  Format::v2int8S};
        ef constexpr (std::is_same_v<T, vec2<i16>>) return {4,   4,  Format::v2int16S};
        ef constexpr (std::is_same_v<T, vec2<i32>>) return {8,   8,  Format::v2int32S};
        ef constexpr (std::is_same_v<T, vec2<i64>>) return {16,  16, Format::v2int64S};

        ef constexpr (std::is_same_v<T, vec2<nu8>>)  return {2,   2,  Format::v2norm8U};
        ef constexpr (std::is_same_v<T, vec2<nu16>>) return {4,   4,  Format::v2norm16U};

        ef constexpr (std::is_same_v<T, vec2<ni8>>)  return {2,   2,  Format::v2norm8S};
        ef constexpr (std::is_same_v<T, vec2<ni16>>) return {4,   4,  Format::v2norm16S};

        ef constexpr (std::is_same_v<T, vec2<f16>>) return {4,   4,  Format::v2float16};
        ef constexpr (std::is_same_v<T, vec2<f32>>) return {8,   8,  Format::v2float32};
        ef constexpr (std::is_same_v<T, vec2<f64>>) return {16,  16, Format::v2float64};


        // vec3
        ef constexpr (std::is_same_v<T, vec3<u8>>)  return {3,   4,  Format::v3int8U};
        ef constexpr (std::is_same_v<T, vec3<u16>>) return {6,   8,  Format::v3int16U};
        ef constexpr (std::is_same_v<T, vec3<u32>>) return {12,  16, Format::v3int32U};
        ef constexpr (std::is_same_v<T, vec3<u64>>) return {24,  32, Format::v3int64U};

        ef constexpr (std::is_same_v<T, vec3<i8>>)  return {3,   4,  Format::v3int8S};
        ef constexpr (std::is_same_v<T, vec3<i16>>) return {6,   8,  Format::v3int16S};
        ef constexpr (std::is_same_v<T, vec3<i32>>) return {12,  16, Format::v3int32S};
        ef constexpr (std::is_same_v<T, vec3<i64>>) return {24,  32, Format::v3int64S};
        
        ef constexpr (std::is_same_v<T, vec3<nu8>>)  return {3,   4,  Format::v3norm8U};
        ef constexpr (std::is_same_v<T, vec3<nu16>>) return {6,   8,  Format::v3norm16U};

        ef constexpr (std::is_same_v<T, vec3<ni8>>)  return {3,   4,  Format::v3norm8S};
        ef constexpr (std::is_same_v<T, vec3<ni16>>) return {6,   8,  Format::v3norm16S};

        ef constexpr (std::is_same_v<T, vec3<f16>>) return {6,   8,  Format::v3float16};
        ef constexpr (std::is_same_v<T, vec3<f32>>) return {12,  16, Format::v3float32};
        ef constexpr (std::is_same_v<T, vec3<f64>>) return {24,  32, Format::v3float64};


        // vec4
        ef constexpr (std::is_same_v<T, vec4<u8>>)  return {4,   4,  Format::v4int8U};
        ef constexpr (std::is_same_v<T, vec4<u16>>) return {8,   8,  Format::v4int16U};
        ef constexpr (std::is_same_v<T, vec4<u32>>) return {16,  16, Format::v4int32U};
        ef constexpr (std::is_same_v<T, vec4<u64>>) return {32,  32, Format::v4int64U};

        ef constexpr (std::is_same_v<T, vec4<i8>>)  return {4,   4,  Format::v4int8S};
        ef constexpr (std::is_same_v<T, vec4<i16>>) return {8,   8,  Format::v4int16S};
        ef constexpr (std::is_same_v<T, vec4<i32>>) return {16,  16, Format::v4int32S};
        ef constexpr (std::is_same_v<T, vec4<i64>>) return {32,  32, Format::v4int64S};

        ef constexpr (std::is_same_v<T, vec4<nu8>>)  return {4,   4,  Format::v4norm8U};
        ef constexpr (std::is_same_v<T, vec4<nu16>>) return {8,   8,  Format::v4norm16U};

        ef constexpr (std::is_same_v<T, vec4<ni8>>)  return {4,   4,  Format::v4norm8S};
        ef constexpr (std::is_same_v<T, vec4<ni16>>) return {8,   8,  Format::v4norm16S};

        ef constexpr (std::is_same_v<T, vec4<f16>>) return {8,   8,  Format::v4float16};
        ef constexpr (std::is_same_v<T, vec4<f32>>) return {16,  16, Format::v4float32};
        ef constexpr (std::is_same_v<T, vec4<f64>>) return {32,  32, Format::v4float64};


        // mat4x4
        ef constexpr (std::is_same_v<T, mat4x4<f32>>) return {16, 16, Format::v4float32, 4};
        ef constexpr (std::is_same_v<T, mat4x4<f64>>) return {32, 32, Format::v4float64, 4};

        // mat4x3
        ef constexpr (std::is_same_v<T, mat4x3<f32>>) return {16, 16, Format::v4float32, 3};
        ef constexpr (std::is_same_v<T, mat4x3<f64>>) return {32, 32, Format::v4float64, 3};
        else
          static_assert(false, "unsupported type");
      }


      /// @brief Create a list of vt instances for the given types.
      template <typename... Ts>
      static fun make_list() -> std::vector<vt> {
        return { vt::make<Ts>()... };
      }


    private:
      u64 m_size, m_align;
      Format m_format;
      u8 m_count;

    public:
      fun size() const { return m_size; }
      fun align() const { return m_align; }
      fun format() const { return m_format; }
      fun count() const { return m_count; }
  };

}
