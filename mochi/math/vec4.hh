/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include <functional>
#include <type_traits>
#include <utility>
#include <cmath>
#include "mochi/math/vec3.hh"



namespace mochi
{

  /** 
   * @brief A generic 4-dimensional vector.
   * @tparam T The element type.
   */
  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct alignas(sizeof(T)*4) vec4
  {
    public:
      T X{}, Y{}, Z{}, W{};

    public:
      /** @brief Default constructor. */
      inline vec4() {}
      /** @brief Construct with a single broadcasted value. */
      inline vec4(T V): X(V), Y(V), Z(V), W(V) {}
      /** @brief Construct with specific X, Y, Z, and W values. */
      inline vec4(T nX, T nY, T nZ, T nW): X(nX), Y(nY), Z(nZ), W(nW) {}

      /** @brief Construct from another vector of different type. */
      template <typename J>
        requires std::is_arithmetic_v<J>
      inline constexpr vec4(vec4<J> V): X(V.X), Y(V.Y), Z(V.Z), W(V.W) {}

      /** @brief Construct from a 3-dimensional vector and a scalar W. */
      inline constexpr vec4(vec3<T> V, T nW): X(V.X), Y(V.Y), Z(V.Z), W(nW) {}

    public:
      /** @brief Calculate the squared length of the vector. */
      [[nodiscard]] inline constexpr fun dot() const noexcept -> T { return (X*X) + (Y*Y) + (Z*Z) +(W*W); }

      /** @brief Calculate the dot product with another vector. */
      template <typename J>
        requires std::is_arithmetic_v<J>
      [[nodiscard]] inline constexpr fun dot(const vec4<J> &it) const noexcept -> T { return (X*it.X) + (Y*it.Y) + (Z*it.Z) + (W+it.W); }

      /** @brief Calculate the squared distance to another vector. */
      template <typename J>
        requires std::is_arithmetic_v<J>
      [[nodiscard]] inline constexpr fun dist_sq(const vec4<J> &it) const noexcept -> T {
        T dx = X -it.X;
        T dy = Y -it.Y;
        T dz = Z -it.Z;
        T dw = W -it.W;
        return (dx*dx) + (dy*dy) + (dz*dz) + (dw*dw);
      }


      /** @brief Calculate the length of the vector. */
      [[nodiscard]] inline constexpr fun dist() const noexcept -> T { return std::sqrt(this->dot()); }

      /** @brief Calculate the distance to another vector. */
      template <typename J>
        requires std::is_arithmetic_v<J>
      [[nodiscard]] inline constexpr fun dist(const vec4<J> &it) const noexcept -> T { return std::sqrt(this->dist_sq(it)); }
      

      /** @brief Return a normalized copy of this vector. */
      [[nodiscard]] inline constexpr fun normalize() const noexcept -> vec4 {
        T len = dist();
        return vec4(X/len, Y/len, Z/len, W/len);
      }


    public:
      inline constexpr fun operator==(const vec4 &it) const noexcept -> const bool { return (X == it.X && Y == it.Y && Z == it.Z && W == it.W); }
      inline constexpr fun operator!=(const vec4 &it) const noexcept -> const bool { return !(*this == it); }

      inline constexpr fun operator+(const vec4 &it) const -> const vec4 { return vec4(X +it.X, Y +it.Y, Z +it.Z, W +it.W); }
      inline constexpr fun operator-(const vec4 &it) const -> const vec4 { return vec4(X -it.X, Y -it.Y, Z -it.Z, W -it.W); }
      inline constexpr fun operator*(const vec4 &it) const -> const vec4 { return vec4(X *it.X, Y *it.Y, Z *it.Z, W *it.W); }
      inline constexpr fun operator/(const vec4 &it) const -> const vec4 { return vec4(X /it.X, Y /it.Y, Z /it.Z, W /it.W); }

      inline constexpr fun operator+=(const vec4 &It) noexcept -> vec4& { X += It.X; Y += It.Y; Z += It.Z; W += It.W; return *this; }
      inline constexpr fun operator-=(const vec4 &It) noexcept -> vec4& { X -= It.X; Y -= It.Y; Z -= It.Z; W -= It.W; return *this; }
      inline constexpr fun operator*=(const vec4 &It) noexcept -> vec4& { X *= It.X; Y *= It.Y; Z *= It.Z; W *= It.W; return *this; }
      inline constexpr fun operator/=(const vec4 &It) noexcept -> vec4& { X /= It.X; Y /= It.Y; Z /= It.Z; W /= It.W; return *this; }


      inline constexpr fun operator+(const T it) const -> const vec4 { return vec4(X +it, Y +it, Z +it, W +it); }
      inline constexpr fun operator-(const T it) const -> const vec4 { return vec4(X -it, Y -it, Z -it, W -it); }
      inline constexpr fun operator*(const T it) const -> const vec4 { return vec4(X *it, Y *it, Z *it, W *it); }
      inline constexpr fun operator/(const T it) const -> const vec4 { return vec4(X /it, Y /it, Z /it, W /it); }

      inline constexpr fun operator+=(const T It) noexcept -> vec4& { X += It; Y += It; Z += It; W += It; return *this; }
      inline constexpr fun operator-=(const T It) noexcept -> vec4& { X -= It; Y -= It; Z -= It; W -= It; return *this; }
      inline constexpr fun operator*=(const T It) noexcept -> vec4& { X *= It; Y *= It; Z *= It; W *= It; return *this; }
      inline constexpr fun operator/=(const T It) noexcept -> vec4& { X /= It; Y /= It; Z /= It; W /= It; return *this; }
  };

}

template <typename t>
struct std::hash<mochi::vec4<t>> {
  std::size_t operator()(const mochi::vec4<t>& d) const noexcept {
    std::size_t seed = 0;

    auto hash_combine = [&seed](auto &&v) {
      using T = std::decay_t<decltype(v)>;
      std::size_t h;
      
      if constexpr (std::is_enum_v<T>)
        h = std::hash<std::underlying_type_t<T>>{}(std::to_underlying(v));
      else
        h = std::hash<T>{}(v);
      
      seed ^= h + 0x9e3779b97f4a7c15 + (seed << 6) + (seed >> 2);
    };

    hash_combine(d.X);
    hash_combine(d.Y);
    hash_combine(d.Z);
    hash_combine(d.W);

    return seed;
  }
};
