/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/
#pragma once

#include "mochi/basis.hh"
#include <immintrin.h>
#include <xmmintrin.h> // SSE
#include <emmintrin.h> // SSE2



namespace mochi
{

  template <typename T>
  struct SimdTraits;

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

}
