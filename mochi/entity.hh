/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/core/core.hh"
#include <cassert>
#include <vulkan/vulkan_raii.hpp>
#include "entt/entt.hpp"



namespace mochi
{

  struct entity
  {
    public:
      entity(entt::entity handle, core &core)
        : m_handle(handle)
        , m_core(core)
      {}
      

    private:
      entt::entity m_handle = entt::null;
      core &m_core;

    public:
      template<typename T, typename... Args>
      T& add_component(Args&&... args) { return m_core.registry().emplace<T>(m_handle, std::forward<Args>(args)...); }

      template<typename T>
      T& get_component() { return m_core.registry().get<T>(m_handle); }

      template<typename T>
      bool has_component() { return m_core.registry().all_of<T>(m_handle); }
      
      
      void add_child(entity child); 
  };

}
