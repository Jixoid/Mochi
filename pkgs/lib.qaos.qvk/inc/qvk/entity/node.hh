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
#include "qvk/types.hh"
#include <algorithm>
#include <stdexcept>
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace qvk
{
  
  struct node
  {
    public:
      explicit node(node *parent, mat4<f32> model);
      virtual inline ~node() {}
      

    private:
      mat4<f32> m_model;
      std::vector<node*> m_childs;
      node *m_parent{};


    public:
      inline fun getModel() { return m_model; }
      virtual inline fun setModel(mat4<f32> val) -> void { m_model = val; }

      inline fun getWorldPos() -> vec3<f32> {
        auto mat = getModel();
        return vec3<f32>(mat.SwVec[0][3], mat.SwVec[1][3], mat.SwVec[2][3]);
      }

      inline fun& childs() { return m_childs; }

      inline fun getParent() { return m_parent; }
      inline fun setParent(node *val) {
        if (m_parent) {
          auto it = std::find(m_parent->childs().begin(), m_parent->childs().end(), this);

          if (it == m_parent->childs().end())
            throw std::runtime_error("Sahiplenme bozuklu.");

          m_parent->childs().erase(it);
          m_parent = Nil;
        }

        if (val && !m_parent) {
          m_parent = val;
          m_parent->m_childs.push_back(this);
        }
      }

  };

}
