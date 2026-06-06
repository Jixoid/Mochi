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
#include <numbers>
#include "mochi/math/vec3.hh"



namespace mochi
{

  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct [[nodiscard]] quaternion
  {
    public:
      T w{static_cast<T>(1.0)}, x{}, y{}, z{};

    public:
      inline constexpr quaternion() noexcept {}
      inline constexpr quaternion(T nw, T nx, T ny, T nz) noexcept : w(nw), x(nx), y(ny), z(nz) {}

      static inline constexpr fun fromAxisAngle(T angle, vec3<T> axis) noexcept -> quaternion {
        axis = axis.normalize();
        T halfAngle = angle * static_cast<T>(0.5);
        T s = std::sin(halfAngle);

        return quaternion(
          std::cos(halfAngle),
          axis.x * s,
          axis.y * s,
          axis.z * s
        );
      }

      static inline constexpr fun fromTwoVector(vec3<T> u, vec3<T> v) noexcept -> quaternion  {
        u = u.normalize();
        v = v.normalize();
        
        T cosTheta = u.dot(v);
        vec3<T> rotationAxis;

        const T EPS = static_cast<T>(1e-6);
        if (cosTheta < static_cast<T>(-1.0) + EPS) {
          rotationAxis = vec3<T>(0, 1, 0).cross(u);
          if (rotationAxis.length() < static_cast<T>(0.01))
            rotationAxis = vec3<T>(1, 0, 0).cross(u);
          return fromAxisAngle(std::numbers::pi_v<T>, rotationAxis.normalize());
        }

        rotationAxis = u.cross(v);
        T s = std::sqrt((static_cast<T>(1.0) + cosTheta) * static_cast<T>(2.0));
        T invS = static_cast<T>(1.0) / s;

        return quaternion(
          s * static_cast<T>(0.5),
          rotationAxis.x * invS,
          rotationAxis.y * invS,
          rotationAxis.z * invS
        ).normalize();
      }

      static inline constexpr fun lookAt(const vec3<T> &pos, const vec3<T> &target, const vec3<T> &up) noexcept -> quaternion  {
        vec3<T> forward = (target - pos).normalize();
        
        vec3<T> right = forward.cross(up).normalize();
        vec3<T> orthoUp = right.cross(forward).normalize();
        
        quaternion q1 = fromTwoVector(vec3<T>(0, 0, -1), forward);
        vec3<T> currentUp = q1.rotate(vec3<T>(0, 1, 0));
        quaternion q2 = fromTwoVector(currentUp, orthoUp);
        
        return q2 * q1;
      }

      static inline constexpr fun slerp(quaternion q1, quaternion q2, T t) noexcept -> quaternion  {
        q1 = q1.normalize();
        q2 = q2.normalize();

        T dot = (q1.w * q2.w) + (q1.x * q2.x) + (q1.y * q2.y) + (q1.z * q2.z);

        if (dot < static_cast<T>(0.0)) {
          q2 = quaternion(-q2.w, -q2.x, -q2.y, -q2.z);
          dot = -dot;
        }

        if (dot > static_cast<T>(0.9995)) {
          return quaternion(
            q1.w + t * (q2.w - q1.w),
            q1.x + t * (q2.x - q1.x),
            q1.y + t * (q2.y - q1.y),
            q1.z + t * (q2.z - q1.z)
          ).normalize();
        }

        T theta_0 = std::acos(dot);
        T theta = theta_0 * t;
        T sin_theta_0 = std::sin(theta_0);
        T sin_theta = std::sin(theta);

        T s0 = std::cos(theta) - dot * sin_theta / sin_theta_0;
        T s1 = sin_theta / sin_theta_0;

        return quaternion(
          (s0 * q1.w) + (s1 * q2.w),
          (s0 * q1.x) + (s1 * q2.x),
          (s0 * q1.y) + (s1 * q2.y),
          (s0 * q1.z) + (s1 * q2.z)
        );
      }


    public:
      inline constexpr fun operator*(const quaternion &it) const noexcept -> quaternion {
        return quaternion(
          w * it.w - x * it.x - y * it.y - z * it.z,
          w * it.x + x * it.w + y * it.z - z * it.y,
          w * it.y - x * it.z + y * it.w + z * it.x,
          w * it.z + x * it.y - y * it.x + z * it.w
        );
      }

      
    public:
      [[nodiscard]] inline constexpr fun rotate(const vec3<T> &v) const noexcept -> vec3<T> {
        vec3<T> u{x, y, z};
        vec3<T> t = u.cross(v) * static_cast<T>(2.0); 
        
        return v + (t * w) + u.cross(t);
      }

      [[nodiscard]] inline constexpr fun conjugate() const noexcept -> quaternion {
        return quaternion(w, -x, -y, -z);
      }

      [[nodiscard]] inline constexpr fun normalize() const noexcept -> quaternion {
        T len = std::sqrt(w*w + x*x + y*y + z*z);
        if (len > static_cast<T>(0.0)) {
          T invLen = static_cast<T>(1.0) / len;
          return quaternion(w*invLen, x*invLen, y*invLen, z*invLen);
        }
        return quaternion();
      }
    
  };



  
  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr auto& get(quaternion<T> &q) noexcept {
    static_assert(i < 4, "index out of bounds for quaternion");
    if constexpr (i == 0) return q.w;
    else if constexpr (i == 1) return q.x;
    else if constexpr (i == 2) return q.y;
    else return q.z;
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr const auto& get(const quaternion<T> &q) noexcept {
    static_assert(i < 4, "index out of bounds for quaternion");
    if constexpr (i == 0) return q.w;
    else if constexpr (i == 1) return q.x;
    else if constexpr (i == 2) return q.y;
    else return q.z;
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr auto&& get(quaternion<T> &&q) noexcept {
    static_assert(i < 4, "index out of bounds for quaternion");
    if constexpr (i == 0) return std::move(q.w);
    else if constexpr (i == 1) return std::move(q.x);
    else if constexpr (i == 2) return std::move(q.y);
    else return std::move(q.z);
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr const auto&& get(const quaternion<T> &&q) noexcept {
    static_assert(i < 4, "index out of bounds for quaternion");
    if constexpr (i == 0) return std::move(q.w);
    else if constexpr (i == 1) return std::move(q.x);
    else if constexpr (i == 2) return std::move(q.y);
    else return std::move(q.z);
  }
}

namespace std {
  template <typename T>
  struct tuple_size<mochi::quaternion<T>> : std::integral_constant<std::size_t, 4> {};

  template <std::size_t i, typename T>
  struct tuple_element<i, mochi::quaternion<T>> {
    static_assert(i < 4, "index out of bounds for quaternion");
    using type = T;
  };

  template <typename T>
  struct formatter<mochi::quaternion<T>> {
    constexpr auto parse(std::format_parse_context& ctx) { return ctx.begin(); }
    auto format(const mochi::quaternion<T>& q, std::format_context& ctx) const {
      return std::format_to(ctx.out(), "quaternion{{w: {}, x: {}, y: {}, z: {}}}", q.w, q.x, q.y, q.z);
    }
  };
}

template <typename T>
inline fun operator<<(std::ostream &os, const mochi::quaternion<T> &q) -> std::ostream& {
  return os << std::format("quaternion{{w: {}, x: {}, y: {}, z: {}}}", q.w, q.x, q.y, q.z);
}
