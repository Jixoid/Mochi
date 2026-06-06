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
#include "mochi/math/mat4x4.hh"
#include "mochi/math/vec4.hh"
#include "mochi/math/vec3.hh"
#include "mochi/math/quaternion.hh"
#include "mochi/math/simd_traits.hh"



namespace mochi
{

  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct alignas(alignof(vec<T, 4>)) mat4x3
  {
    public:
      union {
        T SwVec[3][4];
        vec4<T> SwVec4[3];
        vec<T,4> HwVec[3];
      };

      using ST = SimdTraits<T>;
      using reg_t = typename ST::reg_t;


    public:
      inline mat4x3(): SwVec4{
        vec4<T>(1,0,0,0),
        vec4<T>(0,1,0,0),
        vec4<T>(0,0,1,0)
      } {}

      inline mat4x3(vec4<T> R1, vec4<T> R2, vec4<T> R3): SwVec4{R1,R2,R3} {}


    public:
      inline fun operator*(const mat4x3 &it) const -> mat4x3 {
        mat4x3 result;
        for (int i{}; i < 3; i++)
        {
          vec<T, 4> x(SwVec[i][0]);
          vec<T, 4> y(SwVec[i][1]);
          vec<T, 4> z(SwVec[i][2]);

          auto r = x * it.HwVec[0];
          r = (y * it.HwVec[1]) + r;
          r = (z * it.HwVec[2]) + r;
          
          vec<T, 4> last_row_scaled({0, 0, 0, SwVec[i][3]});
          r = r + last_row_scaled;

          result.HwVec[i] = r;
        }
        return result;
      }

      inline fun operator*(const vec4<T> &v) const -> vec4<T> {
        return vec4<T>(
          SwVec[0][0] * v.x + SwVec[0][1] * v.y + SwVec[0][2] * v.z + SwVec[0][3] * v.w,
          SwVec[1][0] * v.x + SwVec[1][1] * v.y + SwVec[1][2] * v.z + SwVec[1][3] * v.w,
          SwVec[2][0] * v.x + SwVec[2][1] * v.y + SwVec[2][2] * v.z + SwVec[2][3] * v.w,
          v.w
        );
      }

      inline fun operator*(const vec3<T> &v) const -> vec3<T> {
        return vec3<T>(
          SwVec[0][0] * v.x + SwVec[0][1] * v.y + SwVec[0][2] * v.z + SwVec[0][3],
          SwVec[1][0] * v.x + SwVec[1][1] * v.y + SwVec[1][2] * v.z + SwVec[1][3],
          SwVec[2][0] * v.x + SwVec[2][1] * v.y + SwVec[2][2] * v.z + SwVec[2][3]
        );
      }


    public:
      inline fun inverse() const -> mat4x3 {
        T c00 = SwVec[1][1] * SwVec[2][2] - SwVec[1][2] * SwVec[2][1];
        T c01 = SwVec[1][2] * SwVec[2][0] - SwVec[1][0] * SwVec[2][2];
        T c02 = SwVec[1][0] * SwVec[2][1] - SwVec[1][1] * SwVec[2][0];
        
        T c10 = SwVec[0][2] * SwVec[2][1] - SwVec[0][1] * SwVec[2][2];
        T c11 = SwVec[0][0] * SwVec[2][2] - SwVec[0][2] * SwVec[2][0];
        T c12 = SwVec[0][1] * SwVec[2][0] - SwVec[0][0] * SwVec[2][1];
        
        T c20 = SwVec[0][1] * SwVec[1][2] - SwVec[0][2] * SwVec[1][1];
        T c21 = SwVec[0][2] * SwVec[1][0] - SwVec[0][0] * SwVec[1][2];
        T c22 = SwVec[0][0] * SwVec[1][1] - SwVec[0][1] * SwVec[1][0];

        T det = SwVec[0][0] * c00 + SwVec[0][1] * c01 + SwVec[0][2] * c02;

        const T EPS = static_cast<T>(1e-12);
        if (std::abs(det) < EPS) return mat4x3();

        T invDet = static_cast<T>(1.0) / det;

        T r00 = c00 * invDet;
        T r01 = c10 * invDet;
        T r02 = c20 * invDet;
        
        T r10 = c01 * invDet;
        T r11 = c11 * invDet;
        T r12 = c21 * invDet;
        
        T r20 = c02 * invDet;
        T r21 = c12 * invDet;
        T r22 = c22 * invDet;

        T tx = SwVec[0][3];
        T ty = SwVec[1][3];
        T tz = SwVec[2][3];

        T ntx = -(r00 * tx + r01 * ty + r02 * tz);
        T nty = -(r10 * tx + r11 * ty + r12 * tz);
        T ntz = -(r20 * tx + r21 * ty + r22 * tz);

        return mat4x3(
          {r00, r01, r02, ntx},
          {r10, r11, r12, nty},
          {r20, r21, r22, ntz}
        );
      }


    public:
      static inline fun zero() -> mat4x3 {
        mat4x3 res;
        res.SwVec4[0] = res.SwVec4[1] = res.SwVec4[2] = vec4<T>();
        return res;
      }

      static inline fun model(vec3<T> position, quaternion<T> rotate, vec3<T> scale) -> mat4x3 {
        auto q = rotate.normalize();

        T xx = q.x * q.x;
        T yy = q.y * q.y;
        T zz = q.z * q.z;
        T xy = q.x * q.y;
        T xz = q.x * q.z;
        T yz = q.y * q.z;
        T wx = q.w * q.x;
        T wy = q.w * q.y;
        T wz = q.w * q.z;

        const T one = static_cast<T>(1.0);
        const T two = static_cast<T>(2.0);

        return mat4x3(
          vec4<T>(
            (one - two * (yy + zz)) * scale.x,
            (two * (xy - wz)) * scale.y,
            (two * (xz + wy)) * scale.z,
            position.x
          ),
          vec4<T>(
            (two * (xy + wz)) * scale.x,
            (one - two * (xx + zz)) * scale.y,
            (two * (yz - wx)) * scale.z,
            position.y
          ),
          vec4<T>(
            (two * (xz - wy)) * scale.x,
            (two * (yz + wx)) * scale.y,
            (one - two * (xx + yy)) * scale.z,
            position.z
          )
        );
      }


    public:
      inline operator mat4x4<T>() {
        return mat4x4<T>(
          SwVec4[0],
          SwVec4[1],
          SwVec4[2],
          vec4<T>{0,0,0,1}
        );
      }

  };




  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr fun get(mat4x3<T> &m) noexcept -> vec4<T>& {
    static_assert(i < 3, "index out of bounds for mat4x3");
    return m.SwVec4[i];
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr fun get(const mat4x3<T> &m) noexcept -> const vec4<T>& {
    static_assert(i < 3, "index out of bounds for mat4x3");
    return m.SwVec4[i];
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr fun get(mat4x3<T> &&m) noexcept -> vec4<T>&& {
    static_assert(i < 3, "index out of bounds for mat4x3");
    return std::move(m.SwVec4[i]);
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr fun get(const mat4x3<T> &&m) noexcept -> const vec4<T>&& {
    static_assert(i < 3, "index out of bounds for mat4x3");
    return std::move(m.SwVec4[i]);
  }
}

namespace std {
  template <typename T>
  struct tuple_size<mochi::mat4x3<T>> : std::integral_constant<std::size_t, 3> {};

  template <std::size_t i, typename T>
  struct tuple_element<i, mochi::mat4x3<T>> {
    static_assert(i < 3, "index out of bounds for mat4x3");
    using type = mochi::vec4<T>;
  };

  template <typename T>
  struct formatter<mochi::mat4x3<T>> {
    constexpr auto parse(std::format_parse_context& ctx) { return ctx.begin(); }
    auto format(const mochi::mat4x3<T>& m, std::format_context& ctx) const {
      // vec4 formatter will automatically be invoked here
      return std::format_to(ctx.out(), "mat4x3{{{}, {}, {}}}", m.SwVec4[0], m.SwVec4[1], m.SwVec4[2]);
    }
  };
}


template <typename T>
inline fun operator<<(std::ostream &os, const mochi::mat4x3<T> &m) -> std::ostream& {
  return os << std::format("mat4x3{{{}, {}, {}}}", m.SwVec4[0], m.SwVec4[1], m.SwVec4[2]);
}
