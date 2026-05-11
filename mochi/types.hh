/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once


#include "mochi/basis.hh"
#include <string_view>
#include <type_traits>

#define ef else if



namespace mochi
{

  struct core;
  

  namespace module
  {
    struct bridge;
    struct display;
    struct device;
    struct renderer;
    struct memory;
  }


  namespace rhi
  {
    struct slotPush;
    struct slotDesc;
    struct slotVertex;
    struct descset;

    struct pipeline;
    struct buffer;
    struct shader;

    /** @brief Information structure for a type. */
    template <typename T>
    struct info;
  }


  namespace asset
  {
    struct mesh;
    struct texture2;
  }




  /// Geometry
  
  /** @brief 2D Vector structure. */
  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct vec2;

  /** @brief 3D Vector structure. */
  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct vec3;

  /** @brief 4D Vector structure. */
  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct vec4;

  /** @brief Quaternion structure. */
  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct quaternion;

  /** @brief 4x4 Matrix structure. */
  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct mat4;




  /// Mapped Memory
  
  /** 
   * @brief Helper structure for memory-mapped files. 
   */
  struct mappedFile
  {
    public:
      /** 
       * @brief Construct a new mappedFile object.
       * @param fpath The path to the file to map.
       */
      explicit mappedFile(std::string fpath);

      /** @brief Destroy the mappedFile object and unmap memory. */
      ~mappedFile();

    public:
      /**
       * @brief Get a string view of the mapped file data.
       * @return std::string_view View of the file data.
       */
      inline fun view() const -> std::string_view {
        return {static_cast<const char*>(data), size};
      }

    private:
      #if defined(__unix__) || defined(__APPLE__)
      int fd{-1};
      void *data{(void*)(-1)};
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
