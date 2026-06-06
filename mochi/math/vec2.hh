/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/
#pragma once

#include "mochi/basis.hh"
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
  struct [[nodiscard]] alignas(sizeof(T)*2) vec2
  {
    public:
      inline constexpr vec2() {}
      inline constexpr vec2(T v): x(v), y(v) {}
      inline constexpr vec2(T x, T y): x(x), y(y) {}

      template <typename J>
        requires (std::is_arithmetic_v<J> || is_norm_v<J> || std::is_same_v<J, f16>)
      inline explicit constexpr vec2(vec2<J> v): x(v.x), y(v.y) {}


    public:
      T x{}, y{};


    public:
      [[nodiscard]] inline constexpr fun dot() const noexcept -> T { return (x*x) + (y*y); }

      [[nodiscard]] inline constexpr fun dot(const vec2 &it) const noexcept -> T { return (x*it.x) + (y*it.y); }



      [[nodiscard]] inline constexpr fun dist_sq(const vec2 &it) const noexcept -> T {
        const T dx = x -it.x;
        const T dy = y -it.y;
        return (dx*dx) + (dy*dy);
      }

      [[nodiscard]] inline fun length() const noexcept -> T { return std::sqrt(dot()); }
      
      [[nodiscard]] inline fun dist(const vec2 &it) const noexcept -> T { return std::sqrt(dist_sq(it)); }


      [[nodiscard]] inline constexpr fun cross(const vec2 &it) const noexcept -> T { return (x*it.y) - (y*it.x); }


      inline constexpr fun normalize() const noexcept -> vec2 {
        const T len = length();
        return vec2(x/len, y/len);
      }

      inline constexpr fun lerp(const vec2 &it, T t) const noexcept -> vec2 {
        return vec2(x +(it.x-x) *t, y +(it.y-y) *t);
      }
      
      inline constexpr fun clamp(const vec2 &min, const vec2 &max) const noexcept -> vec2 {
        return vec2(
          std::clamp(x, min.x, max.x),
          std::clamp(y, min.y, max.y)
        );
      }

      inline constexpr fun min(const vec2 &it) const noexcept -> vec2 {
        return vec2(std::min(x, it.x), std::min(y, it.y));
      }

      inline constexpr fun max(const vec2 &it) const noexcept -> vec2 {
        return vec2(std::max(x, it.x), std::max(y, it.y));
      }

      inline constexpr fun reflect(const vec2 &normal) const noexcept -> vec2 {
        return *this - (normal * static_cast<T>(2) * dot(normal));
      }

      inline constexpr fun project(const vec2 &it) const noexcept -> vec2 {
        const T d = it.dot();
        if (d == static_cast<T>(0)) return vec2(0);
        return it * (dot(it) / d);
      }

      inline constexpr fun reject(const vec2 &it) const noexcept -> vec2 {
        return *this - project(it);
      }

      inline fun angle_between(const vec2 &it) const noexcept -> T {
        return std::acos(std::clamp(dot(it) / (length() * it.length()), static_cast<T>(-1), static_cast<T>(1)));
      }

      inline fun rotate(T angle_radians) const noexcept -> vec2 {
        const T c = std::cos(angle_radians);
        const T s = std::sin(angle_radians);
        return vec2(x * c - y * s, x * s + y * c);
      }


    public:
      bool operator==(const vec2&) const = default;
      bool operator!=(const vec2&) const = default;

      inline constexpr fun operator-() const noexcept -> vec2 { return vec2(-x, -y); }

      inline constexpr fun operator+(const vec2 &it) const noexcept -> vec2 { return vec2(x +it.x, y +it.y); }
      inline constexpr fun operator-(const vec2 &it) const noexcept -> vec2 { return vec2(x -it.x, y -it.y); }
      inline constexpr fun operator*(const vec2 &it) const noexcept -> vec2 { return vec2(x *it.x, y *it.y); }
      inline constexpr fun operator/(const vec2 &it) const noexcept -> vec2 { return vec2(x /it.x, y /it.y); }

      inline constexpr fun operator+=(const vec2 &it) noexcept -> vec2& { x += it.x; y += it.y; return *this; }
      inline constexpr fun operator-=(const vec2 &it) noexcept -> vec2& { x -= it.x; y -= it.y; return *this; }
      inline constexpr fun operator*=(const vec2 &it) noexcept -> vec2& { x *= it.x; y *= it.y; return *this; }
      inline constexpr fun operator/=(const vec2 &it) noexcept -> vec2& { x /= it.x; y /= it.y; return *this; }


      inline constexpr fun operator+(const T it) const noexcept -> vec2 { return vec2(x +it, y +it); }
      inline constexpr fun operator-(const T it) const noexcept -> vec2 { return vec2(x -it, y -it); }
      inline constexpr fun operator*(const T it) const noexcept -> vec2 { return vec2(x *it, y *it); }
      inline constexpr fun operator/(const T it) const noexcept -> vec2 { return vec2(x /it, y /it); }

      inline constexpr fun operator+=(const T it) noexcept -> vec2& { x += it; y += it; return *this; }
      inline constexpr fun operator-=(const T it) noexcept -> vec2& { x -= it; y -= it; return *this; }
      inline constexpr fun operator*=(const T it) noexcept -> vec2& { x *= it; y *= it; return *this; }
      inline constexpr fun operator/=(const T it) noexcept -> vec2& { x /= it; y /= it; return *this; }


    public:
      inline constexpr fun operator[](u0 idx) -> T& {
        assert(idx < 2 && "index out of bounds for vec2.");
        switch (idx) {
          case 0: return x;
          case 1: return y;
          default: std::unreachable();
        }
      }

      inline constexpr fun operator[](u0 idx) const -> const T& {
        assert(idx < 2 && "index out of bounds for vec2.");
        switch (idx) {
          case 0: return x;
          case 1: return y;
          default: std::unreachable();
        }
      }

    public:
      static inline constexpr fun zero() noexcept -> vec2 { return vec2(0); }
      static inline constexpr fun one() noexcept -> vec2 { return vec2(1); }
      static inline constexpr fun up() noexcept -> vec2 { return vec2(0, 1); }
      static inline constexpr fun down() noexcept -> vec2 { return vec2(0, -1); }
      static inline constexpr fun left() noexcept -> vec2 { return vec2(-1, 0); }
      static inline constexpr fun right() noexcept -> vec2 { return vec2(1, 0); }

  };




  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr auto& get(vec2<T> &v) noexcept {
    static_assert(i < 2, "index out of bounds for vec2");
    if constexpr (i == 0) return v.x;
    else return v.y;
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr const auto& get(const vec2<T> &v) noexcept {
    static_assert(i < 2, "index out of bounds for vec2");
    if constexpr (i == 0) return v.x;
    else return v.y;
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr auto&& get(vec2<T> &&v) noexcept {
    static_assert(i < 2, "index out of bounds for vec2");
    if constexpr (i == 0) return std::move(v.x);
    else return std::move(v.y);
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr const auto&& get(const vec2<T> &&v) noexcept {
    static_assert(i < 2, "index out of bounds for vec2");
    if constexpr (i == 0) return std::move(v.x);
    else return std::move(v.y);
  }

}

namespace std {
  template <typename T>
  struct tuple_size<mochi::vec2<T>>: std::integral_constant<std::size_t, 2> {};

  template <std::size_t i, typename T>
  struct tuple_element<i, mochi::vec2<T>> {
    static_assert(i < 2, "index out of bounds for vec2");
    using type = T;
  };

  template <typename T>
  struct formatter<mochi::vec2<T>> {
    constexpr auto parse(std::format_parse_context& ctx) { return ctx.begin(); }
    auto format(const mochi::vec2<T>& v, std::format_context& ctx) const {
      return std::format_to(ctx.out(), "vec2{{{}, {}}}", v.x, v.y);
    }
  };
}


template <typename T> inline constexpr auto operator+(const T s, const mochi::vec2<T> &v) -> mochi::vec2<T> { return v+s; }
template <typename T> inline constexpr auto operator-(const T s, const mochi::vec2<T> &v) -> mochi::vec2<T> { return mochi::vec2<T>(s - v.x, s - v.y); }
template <typename T> inline constexpr auto operator*(const T s, const mochi::vec2<T> &v) -> mochi::vec2<T> { return v*s; }
template <typename T> inline constexpr auto operator/(const T s, const mochi::vec2<T> &v) -> mochi::vec2<T> { return mochi::vec2<T>(s / v.x, s / v.y); }


template <typename T>
inline fun operator<<(std::ostream &os, const mochi::vec2<T> &v) -> std::ostream& {
  return os << std::format("vec2{{{}, {}}}",  v.x, v.y);
}
