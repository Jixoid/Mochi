/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/vtype.hh"
#include "mochi/types.hh"
#include <cassert>
#include <cstdlib>
#include <iostream>



namespace mochi::rhi
{

  template<>
  struct info<slotPush>
  {
    public:
      info(vt type): m_type(type) {}


    private:
      vt m_type;
      u32 m_offset{};

    public:
      inline fun  type() const { return m_type; }
      inline fun& offset() { return m_offset; }
  };




  struct slotPush
  {
    public:
      template <typename T>
      inline slotPush(T raw)
        : m_data(
          std::malloc(sizeof(T)),
          sizeof(T)
        )
      {
        T vaw = raw;
        __builtin_memcpy(m_data.ptr(), &vaw, sizeof(T));
        std::cerr << "malloc: " << m_data.ptr() << std::endl;
      }

      inline ~slotPush() {
        std::cerr << "free: " << m_data.ptr() << std::endl;
        std::free(m_data.ptr());
      }

      slotPush(const slotPush&) = delete;
      slotPush& operator=(const slotPush&) = delete;


    private:
      data m_data{};
      
    public:
      inline fun data() const { return m_data; }
  };
 
}
