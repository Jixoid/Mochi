/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "qvk/geometry.hh"



namespace qvk
{

  enum ftype
  {
    ft_wavefront,
  };

  template <ftype T>
  fun read();




  /// Wavefront

  struct wf_index {
    i32 v_idx{-1};  // Pozisyon (Vertex) indeksi
    i32 vt_idx{-1}; // Kaplama (UV / Texture) indeksi
    i32 vn_idx{-1}; // Normal (Yüzey yönü) indeksi
  };

  struct wf_face {
    std::vector<wf_index> vertices; 
  };

  struct wf_obj {
    std::vector<qvk::vec3<f32>> v;  // Pozisyonlar
    std::vector<qvk::vec2<f32>> vt; // UV'ler
    std::vector<qvk::vec3<f32>> vn; // Normaller
    std::vector<wf_face> f;  // Yüzeyler (İndeks bağlantıları)
  };


  template <ftype T>
    requires (T == ftype::ft_wavefront)
  fun read(data) -> wf_obj;

}
