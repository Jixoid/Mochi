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
#include "mochi/math/vec4.hh"
#include "mochi/math/quaternion.hh"

#include <immintrin.h>
#include <xmmintrin.h> // SSE
#include <emmintrin.h> // SSE2



namespace mochi
{

  /** 
   * @brief Traits class for SIMD operations.
   * @tparam T The element type.
   */
  template <typename T>
  struct SimdTraits;

  /** @brief SIMD traits specialization for 64-bit floating point. */
  template <>
  struct SimdTraits<f64> {
    using reg_t = __m256d;
    static inline reg_t load(const f64 *p) { return _mm256_load_pd(p); }
    static inline void store(f64 *p, reg_t r) { _mm256_store_pd(p, r); }
    static inline reg_t set1(f64 v) { return _mm256_set1_pd(v); }
    static inline reg_t mul(reg_t a, reg_t b) { return _mm256_mul_pd(a, b); }
    static inline reg_t sub(reg_t a, reg_t b) { return _mm256_sub_pd(a, b); }
    
    static inline reg_t identity_row(int i) {
      if (i == 0) return _mm256_setr_pd(1.0, 0.0, 0.0, 0.0);
      if (i == 1) return _mm256_setr_pd(0.0, 1.0, 0.0, 0.0);
      if (i == 2) return _mm256_setr_pd(0.0, 0.0, 1.0, 0.0);
      return _mm256_setr_pd(0.0, 0.0, 0.0, 1.0);
    }
  };

  
  /** @brief SIMD traits specialization for 32-bit floating point. */
  template <>
  struct SimdTraits<f32> {
    using reg_t = __m128;

    static inline reg_t load(const f32* p) { return _mm_load_ps(p); }
    static inline void store(f32* p, reg_t r) { _mm_store_ps(p, r); }
    static inline reg_t set1(f32 v) { return _mm_set1_ps(v); }
    static inline reg_t mul(reg_t a, reg_t b) { return _mm_mul_ps(a, b); }
    static inline reg_t sub(reg_t a, reg_t b) { return _mm_sub_ps(a, b); }

    static inline reg_t identity_row(int i) {
      if (i == 0) return _mm_setr_ps(1.0f, 0.0f, 0.0f, 0.0f);
      if (i == 1) return _mm_setr_ps(0.0f, 1.0f, 0.0f, 0.0f);
      if (i == 2) return _mm_setr_ps(0.0f, 0.0f, 1.0f, 0.0f);
      return _mm_setr_ps(0.0f, 0.0f, 0.0f, 1.0f);
    }
  };



  /** 
   * @brief A generic 4x4 matrix, primarily using SIMD for operations.
   * @tparam T The element type.
   */
  template <typename T>
    requires (std::is_arithmetic_v<T> || is_norm_v<T> || std::is_same_v<T, f16>)
  struct mat4
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
      /** @brief Default constructor (Identity matrix). */
      inline mat4(): SwVec4(
        {1,0,0,0},
        {0,1,0,0},
        {0,0,1,0},
        {0,0,0,1}
      ) {}

      /** @brief Construct from 4 row vectors. */
      inline mat4(vec4<T> R1, vec4<T> R2, vec4<T> R3, vec4<T> R4): SwVec4(R1,R2,R3,R4) {}

    
    public:
      inline fun operator*(const mat4 &it) const -> mat4 {
        mat4 result;
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
          SwVec[0][0] * v.X + SwVec[0][1] * v.Y + SwVec[0][2] * v.Z + SwVec[0][3] * v.W,
          SwVec[1][0] * v.X + SwVec[1][1] * v.Y + SwVec[1][2] * v.Z + SwVec[1][3] * v.W,
          SwVec[2][0] * v.X + SwVec[2][1] * v.Y + SwVec[2][2] * v.Z + SwVec[2][3] * v.W,
          SwVec[3][0] * v.X + SwVec[3][1] * v.Y + SwVec[3][2] * v.Z + SwVec[3][3] * v.W
        );
      }


    public:
      inline fun setZero() -> void {
        vec<T, 4> zero{};
        HwVec[0] = HwVec[1] = HwVec[2] = HwVec[3] = zero;
      }

      /**
       * @brief Calculate the inverse of the matrix.
       * @return The inverted matrix.
       */
      inline fun inverse() const -> mat4 {
        mat4 dest;
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

          if (maxv < EPS) return mat4<T>(); // Singular

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

      /**
       * @brief Create a rotation matrix from Euler angles and multiply it with this matrix.
       * @param deg The Euler angles (pitch, yaw, roll) in radians.
       * @return The resulting rotated matrix.
       */
      inline fun rotation(vec3<T> deg) -> mat4 {
        T cx = cosf(deg.X); T sx = sinf(deg.X);
        T cy = cosf(deg.Y); T sy = sinf(deg.Y);
        T cz = cosf(deg.Z); T sz = sinf(deg.Z);

        mat4 res = mat4();

        // Combined Rotation Matrix (Z * Y * X)
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
      /**
       * @brief Create a perspective projection matrix.
       * @param fovDeg Field of view in degrees.
       * @param aspect Aspect ratio (width / height).
       * @param nearZ Near clipping plane.
       * @param farZ Far clipping plane.
       */
      static inline fun perspective(T fovDeg, T aspect, T nearZ, T farZ) -> mat4 {
        mat4 res;
        res.setZero();
        
        T tanHalfFov = std::tan(fovDeg * 0.5 * M_PI / 180.0);
        
        res.SwVec[0][0] = 1.0 / (aspect * tanHalfFov);
        res.SwVec[1][1] = -(1.0 / tanHalfFov);
        res.SwVec[2][2] = -farZ / (farZ - nearZ);
        res.SwVec[2][3] = -(farZ * nearZ) / (farZ - nearZ);
        res.SwVec[3][2] = -1.0;
        res.SwVec[3][3] = 0.0;
        
        return res;
      }

      /**
       * @brief Create a view matrix using a camera position, target, and up vector.
       * @param eye The camera position.
       * @param center The target point to look at.
       * @param up The up direction.
       * @return The view matrix.
       */
      static inline fun lookAt(vec3<T> eye, vec3<T> center, vec3<T> up) -> mat4 {
        // F (Forward): Center - Eye
        auto f = (center-eye).normalize();

        // S (Side): Forward x Up (Cross Product)
        auto s = vec3<T>(
          f.Y*up.Z - f.Z*up.Y,
          f.Z*up.X - f.X*up.Z,
          f.X*up.Y - f.Y*up.X
        ).normalize();

        // U (Re-calculated Up): Side x Forward
        auto u = vec3<T>(
          s.Y*f.Z - s.Z*f.Y,
          s.Z*f.X - s.X*f.Z,
          s.X*f.Y - s.Y*f.X
        );


        mat4 res;
        res.HwVec[0] = {+s.X, +s.Y, +s.Z, 0.0};
        res.HwVec[1] = {+u.X, +u.Y, +u.Z, 0.0};
        res.HwVec[2] = {-f.X, -f.Y, -f.Z, 0.0};
        res.HwVec[3] = {+0.0, +0.0, +0.0, 1.0};

        // Translation
        mat4 translation;
        translation.SwVec[0][3] = -eye.X;
        translation.SwVec[1][3] = -eye.Y;
        translation.SwVec[2][3] = -eye.Z;
        
        res.SwVec[0][3] = -(s.X*eye.X + s.Y*eye.Y + s.Z*eye.Z);
        res.SwVec[1][3] = -(u.X*eye.X + u.Y*eye.Y + u.Z*eye.Z);
        res.SwVec[2][3] =  (f.X*eye.X + f.Y*eye.Y + f.Z*eye.Z);

        return res;
      }

      /**
       * @brief Create a model transformation matrix.
       * @param position The translation vector.
       * @param rotate The rotation quaternion.
       * @param scale The scale vector.
       * @return The model matrix.
       */
      static inline fun model(vec3<T> position, quaternion<T> rotate, vec3<T> scale) -> mat4 {
        auto q = rotate.normalize();

        T xx = q.X * q.X;
        T yy = q.Y * q.Y;
        T zz = q.Z * q.Z;
        T xy = q.X * q.Y;
        T xz = q.X * q.Z;
        T yz = q.Y * q.Z;
        T wx = q.W * q.X;
        T wy = q.W * q.Y;
        T wz = q.W * q.Z;

        // MATRIX ROWS (For Column-Major Math)
        return mat4(
          // Row 0
          {
            (1.0f - 2.0f * (yy + zz)) * scale.X, // R00
            (2.0f * (xy - wz)) * scale.Y,        // R01
            (2.0f * (xz + wy)) * scale.Z,        // R02
            position.X                           // Column 3 (Translation X)
          },

          // Row 1
          {
            (2.0f * (xy + wz)) * scale.X,        // R10
            (1.0f - 2.0f * (xx + zz)) * scale.Y, // R11
            (2.0f * (yz - wx)) * scale.Z,        // R12
            position.Y                           // Column 3 (Translation Y)
          },

          // Row 2
          {
            (2.0f * (xz - wy)) * scale.X,        // R20
            (2.0f * (yz + wx)) * scale.Y,        // R21
            (1.0f - 2.0f * (xx + yy)) * scale.Z, // R22
            position.Z                           // Column 3 (Translation Z)
          },

          // Row 3 (Perspective Slots)
          {
            0.0f,
            0.0f,
            0.0f,
            1.0f
          }
        );
      }

  };

}
