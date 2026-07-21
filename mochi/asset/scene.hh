/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "entt/entt.hpp"


namespace mochi::asset
{

  struct Scene {
    public:
      explicit Scene() = default;

    public:
      static fun make() {
        return make_sptr<Scene>();
      }


    private:
      entt::registry m_regs;

    public:
      fun& regs() { return m_regs; }

      fun create_entity() -> entt::entity { return m_regs.create(); }
      fun destroy_entity(entt::entity entity) -> void { m_regs.destroy(entity); }
  };

}
