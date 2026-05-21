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
#include "mochi/rhi/slotDesc.hh"
#include <cassert>



namespace mochi::rhi
{

  template<>
  struct info<listDesc>
  {
    public:
      info(std::vector<info<slotDesc>> idescs): m_idescs(idescs) {}


    private:
      std::vector<info<slotDesc>> m_idescs;

    public:
      inline fun& idescs() { return m_idescs; }
  };

  

  struct listDesc
  {
    public:
      listDesc(u32 index, std::vector<sptr<slotDesc>> descs): m_index(index), m_descs(descs) {}


    private:
      u32 m_index;
      std::vector<sptr<slotDesc>> m_descs;
    
    public:
      inline fun index() { return m_index; }
      inline fun descs() { return m_descs; }
  };

}
