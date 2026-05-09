/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/entity/node.hh"
#include "mochi/ecs/node.hh"
#include "mochi/core.hh"
#include <algorithm>



namespace mochi::entity
{

  Node::Node(core &eng): m_core(eng) 
  {
    m_entity = m_core.registry().create();
    m_core.registry().emplace<ecs::Node>(m_entity);
  }

  Node::~Node() 
  {
    // Destroys entity and its components
    if (m_core.registry().valid(m_entity)) {
      m_core.registry().destroy(m_entity);
    }
  }


  fun Node::addChild(sptr<Node> child) -> void
  {
    if (auto p = child->getParent())
      p->remChild(child);

    
    child->m_parent = weak_from_this();
    m_children.push_back(child);

    // Update ECS hierarchy component
    auto &hc = m_core.registry().get<ecs::Node>(m_entity);
    hc.children.push_back(child->entity());
    
    auto &child_hc = m_core.registry().get<ecs::Node>(child->entity());
    child_hc.parent = m_entity;
  }

  fun Node::remChild(sptr<Node> child) -> void
  {
    auto it = std::find(m_children.begin(), m_children.end(), child);
    if (it != m_children.end()) 
    {
      child->m_parent.reset();
      m_children.erase(it);

      // Update ECS hierarchy component
      auto &hc = m_core.registry().get<ecs::Node>(m_entity);
      auto ecs_it = std::find(hc.children.begin(), hc.children.end(), child->entity());
      if (ecs_it != hc.children.end()) {
        hc.children.erase(ecs_it);
      }

      auto &child_hc = m_core.registry().get<ecs::Node>(child->entity());
      child_hc.parent = entt::null;
    }
  }

}
