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
#include "mochi/rhi/rhi.hh"

#define ef else if



namespace mochi::rhi
{

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
