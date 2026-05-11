/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/rhi.hh"
#include "mochi/types.hh"
#include <cassert>



namespace mochi::rhi
{

  template<>
  struct info<slotVertex>
  {
    private:
      info(sptr<info<rhi::buffer>> ibuf, VertexInputRate inputRate)
        : m_ibuf(ibuf)
        , m_inputRate(inputRate)
      {}

    public:
      static inline fun make(sptr<info<rhi::buffer>> ibuf, VertexInputRate inputRate) {
        return make_sptr(new info<slotVertex>(ibuf, inputRate));
      }


    private:
      sptr<info<rhi::buffer>> m_ibuf;
      VertexInputRate m_inputRate;

    public:
      inline fun ibuf() { return m_ibuf.get(); }
      inline fun inputRate() { return m_inputRate; }
  };



  struct slotVertex
  {
    public:
      slotVertex(info<slotVertex> *info, rhi::buffer *buf)
        : m_info(info)
        , m_buf(buf)
      {
        assert(info && buf);
      }


    private:
      info<slotVertex> *m_info;
      rhi::buffer *m_buf;
    
    public:
      inline fun info() { return m_info; }
      inline fun buf() { return m_buf; }
  };

}
