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



namespace mochi
{

  /** 
   * @brief A generic 2-dimensional vector.
   * @tparam T The element type.
   */
  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct alignas(sizeof(T)*2) vec2
  {
    public:
      T X{}, Y{};

    public:
      inline constexpr vec2() {}
      /** @brief Construct with a single broadcasted value. */
      inline constexpr vec2(T V): X(V), Y(V) {}
      /** @brief Construct with specific X and Y values. */
      inline constexpr vec2(T nX, T nY): X(nX), Y(nY) {}

      /** @brief Construct from another vector of different type. */
      template <typename J>
        requires std::is_arithmetic_v<J>
      inline constexpr vec2(vec2<J> V): X(V.X), Y(V.Y) {}

    public:
      /** @brief Calculate the squared length of the vector. */
      [[nodiscard]] inline constexpr fun dot() const noexcept -> T { return (X*X) + (Y*Y); }

      /** @brief Calculate the dot product with another vector. */
      template <typename J>
        requires std::is_arithmetic_v<J>
      [[nodiscard]] inline constexpr fun dot(const vec2<J> &it) const noexcept -> T { return (X*it.X) + (Y*it.Y); }


      /** @brief Calculate the squared distance to another vector. */
      template <typename J>
        requires std::is_arithmetic_v<J>
      [[nodiscard]] inline constexpr fun dist_sq(const vec2<J> &it) const noexcept -> T {
        T dx = X -it.X;
        T dy = Y -it.Y;
        return (dx*dx) + (dy*dy);
      }


      /** @brief Calculate the length of the vector. */
      [[nodiscard]] inline constexpr fun dist() const noexcept -> const T { return std::sqrt(this->dot()); }
      
      /** @brief Calculate the distance to another vector. */
      template <typename J>
        requires std::is_arithmetic_v<J>
      [[nodiscard]] inline constexpr fun dist(const vec2<J> &it) const noexcept -> T { return std::sqrt(this->dist_sq(it)); }


      /** @brief Calculate the 2D cross product with another vector. */
      [[nodiscard]] inline constexpr fun cross(const vec2<T>& it) const -> T { return (X*it.Y) - (Y*it.X); }


      /** @brief Return a normalized copy of this vector. */
      [[nodiscard]] inline constexpr fun normalize() const noexcept -> vec2 {
        T len = dist();
        return vec2(X/len, Y/len);
      }
      

    public:
      inline constexpr fun operator==(const vec2 &it) const noexcept -> const bool { return (X == it.X && Y == it.Y); }
      inline constexpr fun operator!=(const vec2 &it) const noexcept -> const bool { return !(*this == it); }

      inline constexpr fun operator+(const vec2 &it) const -> const vec2 { return vec2(X +it.X, Y +it.Y); }
      inline constexpr fun operator-(const vec2 &it) const -> const vec2 { return vec2(X -it.X, Y -it.Y); }
      inline constexpr fun operator*(const vec2 &it) const -> const vec2 { return vec2(X *it.X, Y *it.Y); }
      inline constexpr fun operator/(const vec2 &it) const -> const vec2 { return vec2(X /it.X, Y /it.Y); }

      inline constexpr fun operator+=(const vec2 &It) noexcept -> vec2& { X += It.X; Y += It.Y; return *this; }
      inline constexpr fun operator-=(const vec2 &It) noexcept -> vec2& { X -= It.X; Y -= It.Y; return *this; }
      inline constexpr fun operator*=(const vec2 &It) noexcept -> vec2& { X *= It.X; Y *= It.Y; return *this; }
      inline constexpr fun operator/=(const vec2 &It) noexcept -> vec2& { X /= It.X; Y /= It.Y; return *this; }


      inline constexpr fun operator+(const T it) const -> const vec2 { return vec2(X +it, Y +it); }
      inline constexpr fun operator-(const T it) const -> const vec2 { return vec2(X -it, Y -it); }
      inline constexpr fun operator*(const T it) const -> const vec2 { return vec2(X *it, Y *it); }
      inline constexpr fun operator/(const T it) const -> const vec2 { return vec2(X /it, Y /it); }

      inline constexpr fun operator+=(const T It) noexcept -> vec2& { X += It; Y += It; return *this; }
      inline constexpr fun operator-=(const T It) noexcept -> vec2& { X -= It; Y -= It; return *this; }
      inline constexpr fun operator*=(const T It) noexcept -> vec2& { X *= It; Y *= It; return *this; }
      inline constexpr fun operator/=(const T It) noexcept -> vec2& { X /= It; Y /= It; return *this; }
  };

}

template <typename t>
struct std::hash<mochi::vec2<t>> {
  std::size_t operator()(const mochi::vec2<t>& d) const noexcept {
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

    return seed;
  }
};
