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
#include <utility>
#include <cmath>
#include <numbers>
#include "mochi/math/vec4.hh"
#include "mochi/math/simd_traits.hh"



namespace mochi
{

  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct mat4x4
  {
    public:
      union {
        T SwVec[4][4];
        vec4<T> SwVec4[4];
        vec<T,4> HwVec[4];
      };

      using ST = SimdTraits<T>;
      using reg_t = typename ST::reg_t;


    public:
      inline mat4x4(): SwVec4{
        vec4<T>(1,0,0,0),
        vec4<T>(0,1,0,0),
        vec4<T>(0,0,1,0),
        vec4<T>(0,0,0,1)
      } {}

      inline mat4x4(vec4<T> R1, vec4<T> R2, vec4<T> R3, vec4<T> R4): SwVec4{R1,R2,R3,R4} {}

    
    public:
      inline fun operator*(const mat4x4 &it) const -> mat4x4 {
        mat4x4 result;
        for (int i{}; i < 4; i++)
        {
          vec<T, 4> x(SwVec[i][0]);
          vec<T, 4> y(SwVec[i][1]);
          vec<T, 4> z(SwVec[i][2]);
          vec<T, 4> w(SwVec[i][3]);

          auto r = x * it.HwVec[0];
          r = (y * it.HwVec[1]) + r;
          r = (z * it.HwVec[2]) + r;
          r = (w * it.HwVec[3]) + r;

          result.HwVec[i] = r;
        }
        return result;
      }

      inline fun operator*(const vec4<T> &v) const -> vec4<T> {
        return vec4<T>(
          SwVec[0][0] * v.x + SwVec[0][1] * v.y + SwVec[0][2] * v.z + SwVec[0][3] * v.w,
          SwVec[1][0] * v.x + SwVec[1][1] * v.y + SwVec[1][2] * v.z + SwVec[1][3] * v.w,
          SwVec[2][0] * v.x + SwVec[2][1] * v.y + SwVec[2][2] * v.z + SwVec[2][3] * v.w,
          SwVec[3][0] * v.x + SwVec[3][1] * v.y + SwVec[3][2] * v.z + SwVec[3][3] * v.w
        );
      }


    public:
      inline fun inverse() const -> mat4x4 {
        mat4x4 dest;
        reg_t rows[4], arows[4];

        for (int i = 0; i < 4; ++i) {
          rows[i] = ST::load(SwVec[i]);
          arows[i] = ST::identity_row(i);
        }

        T tmp alignas(32) [8];
        const T EPS = static_cast<T>(1e-12);

        for (int k = 0; k < 4; k++) {
          int pivot = k;
          T maxv = 0;
          for (int i = k; i < 4; i++) {
            ST::store(tmp, rows[i]);
            T val = std::abs(tmp[k]);
            if (val > maxv) { maxv = val; pivot = i; }
          }

          if (maxv < EPS) return mat4x4<T>();

          if (pivot != k) {
            std::swap(rows[k], rows[pivot]);
            std::swap(arows[k], arows[pivot]);
          }

          ST::store(tmp, rows[k]);
          T pv = tmp[k];
          reg_t v_inv_pv = ST::set1(static_cast<T>(1.0) / pv);

          rows[k] = ST::mul(rows[k], v_inv_pv);
          arows[k] = ST::mul(arows[k], v_inv_pv);

          for (int i = 0; i < 4; ++i) {
            if (i == k) continue;
            ST::store(tmp, rows[i]);
            T factor = tmp[k];
            if (std::abs(factor) < EPS) continue;

            reg_t vfac = ST::set1(factor);
            rows[i] = ST::sub(rows[i], ST::mul(vfac, rows[k]));
            arows[i] = ST::sub(arows[i], ST::mul(vfac, arows[k]));
          }
        }

        for (int i = 0; i < 4; ++i) ST::store(dest.SwVec[i], arows[i]);
        return dest;
      }

      inline fun rotation(vec3<T> deg) -> mat4x4 {
        T cx = std::cos(deg.x); T sx = std::sin(deg.x);
        T cy = std::cos(deg.y); T sy = std::sin(deg.y);
        T cz = std::cos(deg.z); T sz = std::sin(deg.z);

        mat4x4 res = mat4x4();

        res.SwVec[0][0] = cy * cz;
        res.SwVec[0][1] = sx * sy * cz - cx * sz;
        res.SwVec[0][2] = cx * sy * cz + sx * sz;

        res.SwVec[1][0] = cy * sz;
        res.SwVec[1][1] = sx * sy * sz + cx * cz;
        res.SwVec[1][2] = cx * sy * sz - sx * cz;

        res.SwVec[2][0] = -sy;
        res.SwVec[2][1] = sx * cy;
        res.SwVec[2][2] = cx * cy;

        return (*this)*res;
      }


    public:
      static inline fun zero() -> mat4x4 {
        mat4x4 res;
        res.SwVec4[0] = res.SwVec4[1] = res.SwVec4[2] = res.SwVec4[3] = vec4<T>();

        return res;
      }

      static inline fun perspective(T fovDeg, T aspect, T nearZ, T farZ) -> mat4x4 {
        auto res = mat4x4::zero();
        
        // M_PI yerine modern ve tür güvenli std::numbers::pi_v<T> kullanıldı
        T tanHalfFov = std::tan(fovDeg * static_cast<T>(0.5) * std::numbers::pi_v<T> / static_cast<T>(180.0));
        
        res.SwVec[0][0] = static_cast<T>(1.0) / (aspect * tanHalfFov);
        res.SwVec[1][1] = -(static_cast<T>(1.0) / tanHalfFov);
        res.SwVec[2][2] = -farZ / (farZ - nearZ);
        res.SwVec[2][3] = -(farZ * nearZ) / (farZ - nearZ);
        res.SwVec[3][2] = static_cast<T>(-1.0);
        res.SwVec[3][3] = static_cast<T>(0.0);
        
        return res;
      }

      static inline fun lookAt(vec3<T> eye, vec3<T> center, vec3<T> up) -> mat4x4 {
        auto f = (center-eye).normalize();

        auto s = vec3<T>(
          f.y*up.z - f.z*up.y,
          f.z*up.x - f.x*up.z,
          f.x*up.y - f.y*up.x
        ).normalize();

        auto u = vec3<T>(
          s.y*f.z - s.z*f.y,
          s.z*f.x - s.x*f.z,
          s.x*f.y - s.y*f.x
        );

        mat4x4 res;
        res.HwVec[0] = {+s.x, +s.y, +s.z, static_cast<T>(0.0)};
        res.HwVec[1] = {+u.x, +u.y, +u.z, static_cast<T>(0.0)};
        res.HwVec[2] = {-f.x, -f.y, -f.z, static_cast<T>(0.0)};
        res.HwVec[3] = {static_cast<T>(0.0), static_cast<T>(0.0), static_cast<T>(0.0), static_cast<T>(1.0)};

        res.SwVec[0][3] = -(s.x*eye.x + s.y*eye.y + s.z*eye.z);
        res.SwVec[1][3] = -(u.x*eye.x + u.y*eye.y + u.z*eye.z);
        res.SwVec[2][3] =  (f.x*eye.x + f.y*eye.y + f.z*eye.z);

        return res;
      }

  };




  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr fun get(mat4x4<T> &m) noexcept -> vec4<T>& {
    static_assert(i < 4, "index out of bounds for mat4x4");
    return m.SwVec4[i];
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr fun get(const mat4x4<T> &m) noexcept -> const vec4<T>& {
    static_assert(i < 4, "index out of bounds for mat4x4");
    return m.SwVec4[i];
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr fun get(mat4x4<T> &&m) noexcept -> vec4<T>&& {
    static_assert(i < 4, "index out of bounds for mat4x4");
    return std::move(m.SwVec4[i]);
  }

  template <std::size_t i, typename T>
  [[nodiscard]] inline constexpr fun get(const mat4x4<T> &&m) noexcept -> const vec4<T>&& {
    static_assert(i < 4, "index out of bounds for mat4x4");
    return std::move(m.SwVec4[i]);
  }
}

namespace std {
  template <typename T>
  struct tuple_size<mochi::mat4x4<T>> : std::integral_constant<std::size_t, 4> {};

  template <std::size_t i, typename T>
  struct tuple_element<i, mochi::mat4x4<T>> {
    static_assert(i < 4, "index out of bounds for mat4x4");
    using type = mochi::vec4<T>;
  };

  template <typename T>
  struct formatter<mochi::mat4x4<T>> {
    constexpr auto parse(std::format_parse_context& ctx) { return ctx.begin(); }
    auto format(const mochi::mat4x4<T>& m, std::format_context& ctx) const {
      return std::format_to(ctx.out(), "mat4x4{{{}, {}, {}, {}}}", m.SwVec4[0], m.SwVec4[1], m.SwVec4[2], m.SwVec4[3]);
    }
  };
}


template <typename T>
inline fun operator<<(std::ostream &os, const mochi::mat4x4<T> &m) -> std::ostream& {
  return os << std::format("mat4x4{{{}, {}, {}, {}}}", m.SwVec4[0], m.SwVec4[1], m.SwVec4[2], m.SwVec4[3]);
}
