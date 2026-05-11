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
  struct info<slotDesc>
  {
    private:
      info(sptr<info<buffer>> ibuf, DescriptorType kind, ShaderStageFlags stage)
        : m_ibuf(ibuf)
        , m_kind(kind)
        , m_stage(stage)
      {}

    public:
      static inline fun make(sptr<info<buffer>> ibuf, DescriptorType kind, ShaderStageFlags stage) {
        return make_sptr(new info<slotDesc>(ibuf, kind, stage));
      }


    private:
      sptr<info<buffer>> m_ibuf;
      DescriptorType m_kind;
      ShaderStageFlags m_stage;

    public:
      inline fun ibuf() { return m_ibuf.get(); }
      inline fun kind() { return m_kind; }
      inline fun stage() { return m_stage; }
  };

  

  struct slotDesc
  {
    public:
      slotDesc(info<slotDesc> *info, rhi::buffer *buf)
        : m_info(info)
        , m_buf(buf)
      {
        assert(info && buf);
      }


    private:
      info<slotDesc> *m_info;
      rhi::buffer *m_buf;
    
    public:
      inline fun info() { return m_info; }
      inline fun buf() { return m_buf; }
  };

}
