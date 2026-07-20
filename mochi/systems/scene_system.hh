/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/asset/scene.hh"
#include "mochi/basis.hh"
#include "mochi/types.hh"
#include <cassert>



namespace mochi::sys
{

  struct SceneSystem: noncopy {
    public:
      explicit SceneSystem() = default;
      
    private:
      sptr<asset::Scene> m_activeScene{nil};

    private:
      fun onLoadScene(asset::Scene *scene);

    public:
      fun switchScene(sptr<asset::Scene> new_scene) -> void {
        if (m_activeScene) {}
        m_activeScene = new_scene;
      }

      fun getActiveScene() -> sptr<asset::Scene> {
        return m_activeScene;
      }

  };

}
