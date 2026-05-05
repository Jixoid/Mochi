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

  /** @brief Supported file types for reading. */
  enum ftype
  {
    /** @brief Wavefront OBJ format. */
    ft_wavefront,
  };

  /** 
   * @brief Generic read function template. 
   * @tparam T The file type to read.
   */
  template <ftype T>
  fun read();




  /// Wavefront

  /** @brief Wavefront OBJ indices for a single vertex. */
  struct wf_index {
    /** @brief Position (Vertex) index. */
    i32 v_idx{-1};
    /** @brief Texture (UV) index. */
    i32 vt_idx{-1};
    /** @brief Normal index. */
    i32 vn_idx{-1};
  };

  /** @brief Wavefront OBJ face, consisting of multiple vertices. */
  struct wf_face {
    /** @brief List of indices forming the face. */
    std::vector<wf_index> vertices; 
  };

  /** @brief Complete Wavefront OBJ model data. */
  struct wf_obj {
    /** @brief Vertex positions. */
    std::vector<qvk::vec3<f32>> v;
    /** @brief Texture coordinates (UVs). */
    std::vector<qvk::vec2<f32>> vt;
    /** @brief Vertex normals. */
    std::vector<qvk::vec3<f32>> vn;
    /** @brief Faces (index connections). */
    std::vector<wf_face> f;
  };


  /** 
   * @brief read function for Wavefront OBJ.
   * @tparam T Must be ft_wavefront.
   * @param input Raw data of the OBJ file.
   * @return A parsed Wavefront OBJ model.
   */
  template <ftype T>
    requires (T == ftype::ft_wavefront)
  fun read(data input) -> wf_obj;

}
