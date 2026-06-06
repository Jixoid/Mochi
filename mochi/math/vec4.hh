/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/
#pragma once

#include "mochi/basis.hh"
#include <type_traits>
#include <cmath>
#include "mochi/math/vec2.hh"
#include "mochi/math/vec3.hh"
#include <utility>
#include <algorithm>



namespace mochi
{

  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct [[nodiscard]] alignas(sizeof(T)*4) vec4
  {
    public:
      inline constexpr vec4() {}
      inline constexpr vec4(T v): x(v), y(v), z(v), w(v) {}
      inline constexpr vec4(T x, T y, T z, T w): x(x), y(y), z(z), w(w) {}

      template <typename J>
        requires (std::is_arithmetic_v<J> || is_norm_v<J> || std::is_same_v<J, f16>)
      inline explicit constexpr vec4(vec4<J> v): x(v.x), y(v.y), z(v.z), w(v.w) {}

      inline explicit constexpr vec4(vec2<T> v, T z, T w): x(v.x), y(v.y), z(z), w(w) {}
      inline explicit constexpr vec4(vec3<T> v, T w): x(v.x), y(v.y), z(v.z), w(w) {}


    public:
      T x{}, y{}, z{}, w{};


    public:
      [[nodiscard]] inline constexpr fun dot() const noexcept -> T { return (x*x) + (y*y) + (z*z) +(w*w); }

      [[nodiscard]] inline constexpr fun dot(const vec4 &it) const noexcept -> T { return (x*it.x) + (y*it.y) + (z*it.z) + (w*it.w); }


      [[nodiscard]] inline constexpr fun dist_sq(const vec4 &it) const noexcept -> T {
        const T dx = x -it.x;
        const T dy = y -it.y;
        const T dz = z -it.z;
        const T dw = w -it.w;
        return (dx*dx) + (dy*dy) + (dz*dz) + (dw*dw);
      }

      [[nodiscard]] inline fun length() const noexcept -> T { return std::sqrt(dot()); }

      [[nodiscard]] inline fun dist(const vec4 &it) const noexcept -> T { return std::sqrt(dist_sq(it)); }
      

      inline constexpr fun normalize() const noexcept -> vec4 {
        const T len = length();
        return vec4(x/len, y/len, z/len, w/len);
      }

      inline constexpr fun lerp(const vec4 &it, T t) const noexcept -> vec4 {
        return vec4(x +(it.x-x) *t, y +(it.y-y) *t, z +(it.z-z) *t, w +(it.w-w) *t);
      }

      inline constexpr fun clamp(const vec4 &min, const vec4 &max) const noexcept -> vec4 {
        return vec4(
          std::clamp(x, min.x, max.x),
          std::clamp(y, min.y, max.y),
          std::clamp(z, min.z, max.z),
          std::clamp(w, min.w, max.w)
        );
      }

      inline constexpr fun min(const vec4 &it) const noexcept -> vec4 {
        return vec4(std::min(x, it.x), std::min(y, it.y), std::min(z, it.z), std::min(w, it.w));
      }

      inline constexpr fun max(const vec4 &it) const noexcept -> vec4 {
        return vec4(std::max(x, it.x), std::max(y, it.y), std::max(z, it.z), std::max(w, it.w));
      }

      inline constexpr fun reflect(const vec4 &normal) const noexcept -> vec4 {
        return *this - (normal * static_cast<T>(2) * dot(normal));
      }

      inline constexpr fun xyz() const noexcept -> vec3<T> { return vec3<T>(x, y, z); }
      inline constexpr fun xy() const noexcept -> vec2<T> { return vec2<T>(x, y); }
      inline constexpr fun xz() const noexcept -> vec2<T> { return vec2<T>(x, z); }
      inline constexpr fun yz() const noexcept -> vec2<T> { return vec2<T>(y, z); }
      inline constexpr fun zw() const noexcept -> vec2<T> { return vec2<T>(z, w); }
      inline constexpr fun zwx() const noexcept -> vec3<T> { return vec3<T>(z, w, x); }

      inline constexpr fun project(const vec4 &it) const noexcept -> vec4 {
        const T d = it.dot();
        if (d == static_cast<T>(0)) return vec4(0);
        return it * (dot(it) / d);
      }

      inline constexpr fun reject(const vec4 &it) const noexcept -> vec4 {
        return *this - project(it);
      }


    public:
      bool operator==(const vec4&) const = default;
      bool operator!=(const vec4&) const = default;

      inline constexpr fun operator-() const noexcept -> vec4 { return vec4(-x, -y, -z, -w); }

      inline constexpr fun operator+(const vec4 &it) const noexcept -> vec4 { return vec4(x +it.x, y +it.y, z +it.z, w +it.w); }
      inline constexpr fun operator-(const vec4 &it) const noexcept -> vec4 { return vec4(x -it.x, y -it.y, z -it.z, w -it.w); }
      inline constexpr fun operator*(const vec4 &it) const noexcept -> vec4 { return vec4(x *it.x, y *it.y, z *it.z, w *it.w); }
      inline constexpr fun operator/(const vec4 &it) const noexcept -> vec4 { return vec4(x /it.x, y /it.y, z /it.z, w /it.w); }

      inline constexpr fun operator+=(const vec4 &it) noexcept -> vec4& { x += it.x; y += it.y; z += it.z; w += it.w; return *this; }
      inline constexpr fun operator-=(const vec4 &it) noexcept -> vec4& { x -= it.x; y -= it.y; z -= it.z; w -= it.w; return *this; }
      inline constexpr fun operator*=(const vec4 &it) noexcept -> vec4& { x *= it.x; y *= it.y; z *= it.z; w *= it.w; return *this; }
      inline constexpr fun operator/=(const vec4 &it) noexcept -> vec4& { x /= it.x; y /= it.y; z /= it.z; w /= it.w; return *this; }


      inline constexpr fun operator+(const T it) const noexcept -> vec4 { return vec4(x +it, y +it, z +it, w +it); }
      inline constexpr fun operator-(const T it) const noexcept -> vec4 { return vec4(x -it, y -it, z -it, w -it); }
      inline constexpr fun operator*(const T it) const noexcept -> vec4 { return vec4(x *it, y *it, z *it, w *it); }
      inline constexpr fun operator/(const T it) const noexcept -> vec4 { return vec4(x /it, y /it, z /it, w /it); }

      inline constexpr fun operator+=(const T it) noexcept -> vec4& { x += it; y += it; z += it; w += it; return *this; }
      inline constexpr fun operator-=(const T it) noexcept -> vec4& { x -= it; y -= it; z -= it; w -= it; return *this; }
      inline constexpr fun operator*=(const T it) noexcept -> vec4& { x *= it; y *= it; z *= it; w *= it; return *this; }
      inline constexpr fun operator/=(const T it) noexcept -> vec4& { x /= it; y /= it; z /= it; w /= it; return *this; }


    public:
      inline constexpr fun operator[](u0 idx) -> T& {
        assert(idx < 4 && "index out of bounds for vec4.");
        switch (idx) {
          case 0: return x;
          case 1: return y;
          case 2: return z;
          case 3: return w;
          default: std::unreachable();
        }
      }

      inline constexpr fun operator[](u0 idx) const -> const T& {
        assert(idx < 4 && "index out of bounds for vec4.");
        switch (idx) {
          case 0: return x;
          case 1: return y;
          case 2: return z;
          case 3: return w;
          default: std::unreachable();
        }
      }

    public:
      static inline constexpr fun zero() noexcept -> vec4 { return vec4(0); }
      static inline constexpr fun one() noexcept -> vec4 { return vec4(1); }

  };




  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr auto& get(vec4<T> &v) noexcept {
    static_assert(i < 4, "index out of bounds for vec4");
    if constexpr (i == 0) return v.x;
    else if constexpr (i == 1) return v.y;
    else if constexpr (i == 2) return v.z;
    else return v.w;
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr const auto& get(const vec4<T> &v) noexcept {
    static_assert(i < 4, "index out of bounds for vec4");
    if constexpr (i == 0) return v.x;
    else if constexpr (i == 1) return v.y;
    else if constexpr (i == 2) return v.z;
    else return v.w;
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr auto&& get(vec4<T> &&v) noexcept {
    static_assert(i < 4, "index out of bounds for vec4");
    if constexpr (i == 0) return std::move(v.x);
    else if constexpr (i == 1) return std::move(v.y);
    else if constexpr (i == 2) return std::move(v.z);
    else return std::move(v.w);
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr const auto&& get(const vec4<T> &&v) noexcept {
    static_assert(i < 4, "index out of bounds for vec4");
    if constexpr (i == 0) return std::move(v.x);
    else if constexpr (i == 1) return std::move(v.y);
    else if constexpr (i == 2) return std::move(v.z);
    else return std::move(v.w);
  }
}

namespace std {
  template <typename T>
  struct tuple_size<mochi::vec4<T>>: std::integral_constant<std::size_t, 4> {};

  template <std::size_t i, typename T>
  struct tuple_element<i, mochi::vec4<T>> {
    static_assert(i < 4, "index out of bounds for vec4");
    using type = T;
  };

  template <typename T>
  struct formatter<mochi::vec4<T>> {
    constexpr auto parse(std::format_parse_context& ctx) { return ctx.begin(); }
    auto format(const mochi::vec4<T>& v, std::format_context& ctx) const {
      return std::format_to(ctx.out(), "vec4{{{}, {}, {}, {}}}", v.x, v.y, v.z, v.w);
    }
  };
}


template <typename T> inline constexpr auto operator+(const T s, const mochi::vec4<T> &v) -> mochi::vec4<T> { return v+s; }
template <typename T> inline constexpr auto operator-(const T s, const mochi::vec4<T> &v) -> mochi::vec4<T> { return mochi::vec4<T>(s - v.x, s - v.y, s - v.z, s - v.w); }
template <typename T> inline constexpr auto operator*(const T s, const mochi::vec4<T> &v) -> mochi::vec4<T> { return v*s; }
template <typename T> inline constexpr auto operator/(const T s, const mochi::vec4<T> &v) -> mochi::vec4<T> { return mochi::vec4<T>(s / v.x, s / v.y, s / v.z, s / v.w); }


template <typename T>
inline fun operator<<(std::ostream &os, const mochi::vec4<T> &v) -> std::ostream& {
  return os << std::format("vec4{{{}, {}, {}, {}}}",  v.x, v.y, v.z, v.w);
}
