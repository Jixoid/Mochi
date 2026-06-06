/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/
#pragma once

#include "mochi/basis.hh"
#include "mochi/math/vec2.hh"
#include <cassert>
#include <format>
#include <type_traits>
#include <cmath>
#include <utility>
#include <algorithm>



namespace mochi
{

  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct [[nodiscard]] alignas(sizeof(T)*4) vec3
  {
    public:
      inline constexpr vec3() {}
      inline constexpr vec3(T v): x(v), y(v), z(v) {}
      inline constexpr vec3(T x, T y, T z): x(x), y(y), z(z) {}

      template <typename J>
        requires (std::is_arithmetic_v<J> || is_norm_v<J> || std::is_same_v<J, f16>)
      inline explicit constexpr vec3(vec3<J> v): x(v.x), y(v.y), z(v.z) {}

      inline explicit constexpr vec3(vec2<T> v, T z): x(v.x), y(v.y), z(z) {}


    public:
      T x{}, y{}, z{};


    public:
      [[nodiscard]] inline constexpr fun dot() const noexcept -> T { return (x*x) + (y*y) + (z*z); }

      [[nodiscard]] inline constexpr fun dot(const vec3 &it) const noexcept -> T { return (x*it.x) + (y*it.y) + (z*it.z); }


      [[nodiscard]] inline constexpr fun dist_sq(const vec3 &it) const noexcept -> T {
        const T dx = x -it.x;
        const T dy = y -it.y;
        const T dz = z -it.z;
        return (dx*dx) + (dy*dy) + (dz*dz);
      }

      [[nodiscard]] inline fun length() const noexcept -> T { return std::sqrt(dot()); }

      [[nodiscard]] inline fun dist(const vec3 &it) const noexcept -> T { return std::sqrt(dist_sq(it)); }
      
      inline constexpr fun cross(const vec3 &it) const noexcept -> vec3 {
        return vec3(
          (y*it.z) - (z*it.y),
          (z*it.x) - (x*it.z),
          (x*it.y) - (y*it.x)
        );
      }


      inline constexpr fun normalize() const noexcept -> vec3 {
        const T len = length();
        return vec3(x/len, y/len, z/len);
      }

      inline constexpr fun lerp(const vec3 &it, T t) const noexcept -> vec3 {
        return vec3(x +(it.x-x) *t, y +(it.y-y) *t, z +(it.z-z) *t);
      }

      inline constexpr fun clamp(const vec3 &min, const vec3 &max) const noexcept -> vec3 {
        return vec3(
          std::clamp(x, min.x, max.x),
          std::clamp(y, min.y, max.y),
          std::clamp(z, min.z, max.z)
        );
      }

      inline constexpr fun min(const vec3 &it) const noexcept -> vec3 {
        return vec3(std::min(x, it.x), std::min(y, it.y), std::min(z, it.z));
      }

      inline constexpr fun max(const vec3 &it) const noexcept -> vec3 {
        return vec3(std::max(x, it.x), std::max(y, it.y), std::max(z, it.z));
      }

      inline constexpr fun reflect(const vec3 &normal) const noexcept -> vec3 {
        return *this -(normal * static_cast<T>(2) * dot(normal));
      }

      inline constexpr fun xy() const noexcept -> vec2<T> { return vec2<T>(x, y); }
      inline constexpr fun xz() const noexcept -> vec2<T> { return vec2<T>(x, z); }
      inline constexpr fun yz() const noexcept -> vec2<T> { return vec2<T>(y, z); }

      inline constexpr fun project(const vec3 &it) const noexcept -> vec3 {
        const T d = it.dot();
        if (d == static_cast<T>(0)) return vec3(0);
        return it * (dot(it) / d);
      }

      inline constexpr fun reject(const vec3 &it) const noexcept -> vec3 {
        return *this - project(it);
      }
      

    public:
      bool operator==(const vec3&) const = default;
      bool operator!=(const vec3&) const = default;

      inline constexpr fun operator-() const noexcept -> vec3 { return vec3(-x, -y, -z); }

      inline constexpr fun operator+(const vec3 &it) const noexcept -> vec3 { return vec3(x +it.x, y +it.y, z +it.z); }
      inline constexpr fun operator-(const vec3 &it) const noexcept -> vec3 { return vec3(x -it.x, y -it.y, z -it.z); }
      inline constexpr fun operator*(const vec3 &it) const noexcept -> vec3 { return vec3(x *it.x, y *it.y, z *it.z); }
      inline constexpr fun operator/(const vec3 &it) const noexcept -> vec3 { return vec3(x /it.x, y /it.y, z /it.z); }

      inline constexpr fun operator+=(const vec3 &it) noexcept -> vec3& { x += it.x; y += it.y; z += it.z; return *this; }
      inline constexpr fun operator-=(const vec3 &it) noexcept -> vec3& { x -= it.x; y -= it.y; z -= it.z; return *this; }
      inline constexpr fun operator*=(const vec3 &it) noexcept -> vec3& { x *= it.x; y *= it.y; z *= it.z; return *this; }
      inline constexpr fun operator/=(const vec3 &it) noexcept -> vec3& { x /= it.x; y /= it.y; z /= it.z; return *this; }


      inline constexpr fun operator+(const T it) const noexcept -> vec3 { return vec3(x +it, y +it, z +it); }
      inline constexpr fun operator-(const T it) const noexcept -> vec3 { return vec3(x -it, y -it, z -it); }
      inline constexpr fun operator*(const T it) const noexcept -> vec3 { return vec3(x *it, y *it, z *it); }
      inline constexpr fun operator/(const T it) const noexcept -> vec3 { return vec3(x /it, y /it, z /it); }

      inline constexpr fun operator+=(const T it) noexcept -> vec3& { x += it; y += it; z += it; return *this; }
      inline constexpr fun operator-=(const T it) noexcept -> vec3& { x -= it; y -= it; z -= it; return *this; }
      inline constexpr fun operator*=(const T it) noexcept -> vec3& { x *= it; y *= it; z *= it; return *this; }
      inline constexpr fun operator/=(const T it) noexcept -> vec3& { x /= it; y /= it; z /= it; return *this; }


    public:
      inline constexpr fun operator[](u0 idx) -> T& {
        assert(idx < 3 && "index out of bounds for vec3.");
        switch (idx) {
          case 0: return x;
          case 1: return y;
          case 2: return z;
          default: std::unreachable();
        }
      }

      inline constexpr fun operator[](u0 idx) const -> const T& {
        assert(idx < 3 && "index out of bounds for vec3.");
        switch (idx) {
          case 0: return x;
          case 1: return y;
          case 2: return z;
          default: std::unreachable();
        }
      }

    public:
      static inline constexpr fun zero() noexcept -> vec3 { return vec3(0); }
      static inline constexpr fun one() noexcept -> vec3 { return vec3(1); }
      static inline constexpr fun up() noexcept -> vec3 { return vec3(0, 1, 0); }
      static inline constexpr fun down() noexcept -> vec3 { return vec3(0, -1, 0); }
      static inline constexpr fun left() noexcept -> vec3 { return vec3(-1, 0, 0); }
      static inline constexpr fun right() noexcept -> vec3 { return vec3(1, 0, 0); }
      static inline constexpr fun forward() noexcept -> vec3 { return vec3(0, 0, -1); }
      static inline constexpr fun back() noexcept -> vec3 { return vec3(0, 0, 1); }
      
  };




  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr auto& get(vec3<T> &v) noexcept {
    static_assert(i < 3, "index out of bounds for vec3");
    if constexpr (i == 0) return v.x;
    else if constexpr (i == 1) return v.y;
    else return v.z;
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr const auto& get(const vec3<T> &v) noexcept {
    static_assert(i < 3, "index out of bounds for vec3");
    if constexpr (i == 0) return v.x;
    else if constexpr (i == 1) return v.y;
    else return v.z;
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr auto&& get(vec3<T> &&v) noexcept {
    static_assert(i < 3, "index out of bounds for vec3");
    if constexpr (i == 0) return std::move(v.x);
    else if constexpr (i == 1) return std::move(v.y);
    else return std::move(v.z);
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr const auto&& get(const vec3<T> &&v) noexcept {
    static_assert(i < 3, "Index out of bounds for vec3");
    if constexpr (i == 0) return std::move(v.x);
    else if constexpr (i == 1) return std::move(v.y);
    else return std::move(v.z);
  }
}

namespace std {
  template <typename T>
  struct tuple_size<mochi::vec3<T>>: std::integral_constant<std::size_t, 3> {};

  template <std::size_t i, typename T>
  struct tuple_element<i, mochi::vec3<T>> {
    static_assert(i < 3, "index out of bounds for vec3");
    using type = T;
  };

  template <typename T>
  struct formatter<mochi::vec3<T>> {
    constexpr auto parse(std::format_parse_context& ctx) { return ctx.begin(); }
    auto format(const mochi::vec3<T>& v, std::format_context& ctx) const {
      return std::format_to(ctx.out(), "vec3{{{}, {}, {}}}", v.x, v.y, v.z);
    }
  };
}


template <typename T> inline constexpr auto operator+(const T s, const mochi::vec3<T> &v) -> mochi::vec3<T> { return v+s; }
template <typename T> inline constexpr auto operator-(const T s, const mochi::vec3<T> &v) -> mochi::vec3<T> { return mochi::vec3<T>(s - v.x, s - v.y, s - v.z); }
template <typename T> inline constexpr auto operator*(const T s, const mochi::vec3<T> &v) -> mochi::vec3<T> { return v*s; }
template <typename T> inline constexpr auto operator/(const T s, const mochi::vec3<T> &v) -> mochi::vec3<T> { return mochi::vec3<T>(s / v.x, s / v.y, s / v.z); }


template <typename T>
inline fun operator<<(std::ostream &os, const mochi::vec3<T> &v) -> std::ostream& {
  return os << std::format("vec3{{{}, {}, {}}}",  v.x, v.y, v.z);
}
