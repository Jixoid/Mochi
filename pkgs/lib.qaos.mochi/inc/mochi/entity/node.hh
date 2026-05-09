/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "basis.hh"
#include "mochi/types.hh"
#include "src/entt/entt.hpp"
#include <vector>



namespace mochi::entity
{

  /** @brief Base class for all scene objects. Wraps entt::entity. */
  class Node: public wptr_from<Node>
  {
    public:
      explicit Node(core &eng);
      virtual ~Node();
      

    protected:  
      core &m_core;
      entt::entity m_entity;
      
      wptr<Node> m_parent;
      std::vector<sptr<Node>> m_children;      

    public:
      inline fun  entity() const { return m_entity; }
      inline fun& getCore() const { return m_core; }
      
      virtual fun addChild(sptr<Node> child) -> void;
      virtual fun remChild(sptr<Node> child) -> void;
      
      inline fun  getParent() const -> sptr<Node> { return m_parent.lock(); }
      inline fun& getChildren() const { return m_children; }
  };

}
