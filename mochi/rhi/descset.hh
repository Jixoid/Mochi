/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/types.hh"
#include <cassert>



namespace mochi::rhi
{

  template<>
  struct info<descset>
  {
    private:
      info(std::vector<sptr<info<slotDesc>>> idescs): m_idescs(idescs) {}

    public:
      static inline fun make(std::vector<sptr<info<slotDesc>>> idescs) {
        return make_sptr(new info<descset>(idescs));
      }


    private:
      std::vector<sptr<info<slotDesc>>> m_idescs;

    public:
      inline fun& idescs() { return m_idescs; }
  };

  

  struct descset
  {
    public:
      descset(u32 id, std::vector<sptr<slotDesc>> descs): m_id(id), m_descs(descs) {}


    private:
      u32 m_id;
      std::vector<sptr<slotDesc>> m_descs;
    
    public:
      inline fun id() { return m_id; }
      inline fun descs() { return m_descs; }
  };

}
