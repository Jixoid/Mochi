/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once


#include "Basis.h"
#include "Basis.hh"
#include "vulkan/vulkan.hpp"
#include <string_view>
#include <sys/mman.h>

#define ef else if



namespace qvk
{

  struct core;
  
  struct bridge;
  struct window;
  struct device;
  struct swapchain;
  struct renderer;
  struct memory;



  /// Entities
  struct pipeline;
  struct buffer;
  struct shader;

  struct node;
  struct camera;
  struct light;

  struct mesh;

  template <typename T>
  struct info;

  

  /// Geometry
  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct vec2;

  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct vec3;

  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct vec4;

  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct quaternion;

  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct mat4;




  /// GTypes
  struct gt
  {
    private:
      gt(u64 size, u64 align, vk::Format format, u8 count = 1)
        : m_size(size)
        , m_align(align)
        , m_format(format)
        , m_count(count)
      {}


    public:
      template <typename T>
      static inline fun make() -> gt {
        // primitive
        if constexpr (std::is_same_v<T, u8>)  return {1, 1, vk::Format::eR8Uint};
        ef constexpr (std::is_same_v<T, u16>) return {2, 2, vk::Format::eR16Uint};
        ef constexpr (std::is_same_v<T, u32>) return {4, 4, vk::Format::eR32Uint};
        ef constexpr (std::is_same_v<T, u64>) return {8, 8, vk::Format::eR64Uint};

        ef constexpr (std::is_same_v<T, i8>)  return {1, 1, vk::Format::eR8Sint};
        ef constexpr (std::is_same_v<T, i16>) return {2, 2, vk::Format::eR16Sint};
        ef constexpr (std::is_same_v<T, i32>) return {4, 4, vk::Format::eR32Sint};
        ef constexpr (std::is_same_v<T, i64>) return {8, 8, vk::Format::eR64Sint};

        ef constexpr (std::is_same_v<T, nu8>)  return {1, 1, vk::Format::eR8Unorm};
        ef constexpr (std::is_same_v<T, nu16>) return {2, 2, vk::Format::eR16Unorm};

        ef constexpr (std::is_same_v<T, ni8>)  return {1, 1, vk::Format::eR8Snorm};
        ef constexpr (std::is_same_v<T, ni16>) return {2, 2, vk::Format::eR16Snorm};

        ef constexpr (std::is_same_v<T, f16>) return {2, 2, vk::Format::eR16Sfloat};
        ef constexpr (std::is_same_v<T, f32>) return {4, 4, vk::Format::eR32Sfloat};
        ef constexpr (std::is_same_v<T, f64>) return {8, 8, vk::Format::eR64Sfloat};


        // vec2
        ef constexpr (std::is_same_v<T, vec2<u8>>)  return {2,   2,  vk::Format::eR8G8Uint};
        ef constexpr (std::is_same_v<T, vec2<u16>>) return {4,   4,  vk::Format::eR16G16Uint};
        ef constexpr (std::is_same_v<T, vec2<u32>>) return {8,   8,  vk::Format::eR32G32Uint};
        ef constexpr (std::is_same_v<T, vec2<u64>>) return {16,  16, vk::Format::eR64G64Uint};

        ef constexpr (std::is_same_v<T, vec2<i8>>)  return {2,   2,  vk::Format::eR8G8Sint};
        ef constexpr (std::is_same_v<T, vec2<i16>>) return {4,   4,  vk::Format::eR16G16Sint};
        ef constexpr (std::is_same_v<T, vec2<i32>>) return {8,   8,  vk::Format::eR32G32Sint};
        ef constexpr (std::is_same_v<T, vec2<i64>>) return {16,  16, vk::Format::eR64G64Sint};

        ef constexpr (std::is_same_v<T, vec2<nu8>>)  return {2,   2,  vk::Format::eR8G8Unorm};
        ef constexpr (std::is_same_v<T, vec2<nu16>>) return {4,   4,  vk::Format::eR16G16Unorm};

        ef constexpr (std::is_same_v<T, vec2<ni8>>)  return {2,   2,  vk::Format::eR8G8Snorm};
        ef constexpr (std::is_same_v<T, vec2<ni16>>) return {4,   4,  vk::Format::eR16G16Snorm};

        ef constexpr (std::is_same_v<T, vec2<f16>>) return {4,   4,  vk::Format::eR16G16Sfloat};
        ef constexpr (std::is_same_v<T, vec2<f32>>) return {8,   8,  vk::Format::eR32G32Sfloat};
        ef constexpr (std::is_same_v<T, vec2<f64>>) return {16,  16, vk::Format::eR64G64Sfloat};


        // vec3
        ef constexpr (std::is_same_v<T, vec3<u8>>)  return {3,   3,  vk::Format::eR8G8B8Uint};
        ef constexpr (std::is_same_v<T, vec3<u16>>) return {6,   6,  vk::Format::eR16G16B16Uint};
        ef constexpr (std::is_same_v<T, vec3<u32>>) return {12,  12, vk::Format::eR32G32B32Uint};
        ef constexpr (std::is_same_v<T, vec3<u64>>) return {24,  24, vk::Format::eR64G64B64Uint};

        ef constexpr (std::is_same_v<T, vec3<i8>>)  return {3,   3,  vk::Format::eR8G8B8Sint};
        ef constexpr (std::is_same_v<T, vec3<i16>>) return {6,   6,  vk::Format::eR16G16B16Sint};
        ef constexpr (std::is_same_v<T, vec3<i32>>) return {12,  12, vk::Format::eR32G32B32Sint};
        ef constexpr (std::is_same_v<T, vec3<i64>>) return {24,  24, vk::Format::eR64G64B64Sint};
        
        ef constexpr (std::is_same_v<T, vec3<nu8>>)  return {3,   3,  vk::Format::eR8G8B8Unorm};
        ef constexpr (std::is_same_v<T, vec3<nu16>>) return {6,   6,  vk::Format::eR16G16B16Unorm};

        ef constexpr (std::is_same_v<T, vec3<ni8>>)  return {3,   3,  vk::Format::eR8G8B8Snorm};
        ef constexpr (std::is_same_v<T, vec3<ni16>>) return {6,   6,  vk::Format::eR16G16B16Snorm};

        ef constexpr (std::is_same_v<T, vec3<f16>>) return {6,   6,  vk::Format::eR16G16B16Sfloat};
        ef constexpr (std::is_same_v<T, vec3<f32>>) return {12,  12, vk::Format::eR32G32B32Sfloat};
        ef constexpr (std::is_same_v<T, vec3<f64>>) return {24,  24, vk::Format::eR64G64B64Sfloat};


        // vec4
        ef constexpr (std::is_same_v<T, vec4<u8>>)  return {4,   4,  vk::Format::eR8G8B8A8Uint};
        ef constexpr (std::is_same_v<T, vec4<u16>>) return {8,   8,  vk::Format::eR16G16B16A16Uint};
        ef constexpr (std::is_same_v<T, vec4<u32>>) return {16,  16, vk::Format::eR32G32B32A32Uint};
        ef constexpr (std::is_same_v<T, vec4<u64>>) return {32,  32, vk::Format::eR64G64B64A64Uint};

        ef constexpr (std::is_same_v<T, vec4<i8>>)  return {4,   4,  vk::Format::eR8G8B8A8Sint};
        ef constexpr (std::is_same_v<T, vec4<i16>>) return {8,   8,  vk::Format::eR16G16B16A16Sint};
        ef constexpr (std::is_same_v<T, vec4<i32>>) return {16,  16, vk::Format::eR32G32B32A32Sint};
        ef constexpr (std::is_same_v<T, vec4<i64>>) return {32,  32, vk::Format::eR64G64B64A64Sint};

        ef constexpr (std::is_same_v<T, vec4<nu8>>)  return {4,   4,  vk::Format::eR8G8B8A8Unorm};
        ef constexpr (std::is_same_v<T, vec4<nu16>>) return {8,   8,  vk::Format::eR16G16B16A16Unorm};

        ef constexpr (std::is_same_v<T, vec4<ni8>>)  return {4,   4,  vk::Format::eR8G8B8A8Snorm};
        ef constexpr (std::is_same_v<T, vec4<ni16>>) return {8,   8,  vk::Format::eR16G16B16A16Snorm};

        ef constexpr (std::is_same_v<T, vec4<f16>>) return {8,   8,  vk::Format::eR16G16B16A16Sfloat};
        ef constexpr (std::is_same_v<T, vec4<f32>>) return {16,  16, vk::Format::eR32G32B32A32Sfloat};
        ef constexpr (std::is_same_v<T, vec4<f64>>) return {32,  32, vk::Format::eR64G64B64A64Sfloat};


        // mat4
        ef constexpr (std::is_same_v<T, mat4<f32>>) return {16, 16, vk::Format::eR32G32B32A32Sfloat, 4};
        ef constexpr (std::is_same_v<T, mat4<f64>>) return {32, 32, vk::Format::eR64G64B64A64Sfloat, 4};
        else
          static_assert(false, "unsupported type");
      }


      template <typename... Ts>
      static inline fun make_list() -> std::vector<gt> {
        return { gt::make<Ts>()... };
      }


    private:
      u64 m_size, m_align;
      vk::Format m_format;
      u8 m_count;

    public:
      inline fun size() { return m_size; }
      inline fun align() { return m_align; }
      inline fun format() { return m_format; }
      inline fun count() { return m_count; }
  };



  /// Mapped Memory
  struct mappedFile
  {
    public:
      explicit mappedFile(std::string fpath);

      ~mappedFile();

    public:
      inline fun view() const -> std::string_view {
        return {static_cast<const char*>(data), size};
      }

    private:
      #if defined(__unix__) || defined(__APPLE__)
      int fd{-1};
      void *data{MAP_FAILED};
      #elif defined(_WIN32)
      void* hFile;
      void* hMapping;
      void *data{};
      #endif
      
      u0 size{};

    public:
      inline operator ::data () { return {data, size}; }

  };

}
