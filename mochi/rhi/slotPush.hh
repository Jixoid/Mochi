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
#include "mochi/rhi/vtype.hh"
#include "mochi/types.hh"
#include <cassert>



namespace mochi::rhi
{

  template<>
  struct info<slotPush>
  {
    private:
      info(vt type, ShaderStageFlags shaderStage)
        : m_type(type)
        , m_shaderStage(shaderStage)
      {}

    public:
      static inline fun make(vt type, ShaderStageFlags shaderStage) {
        return make_sptr(new info<slotPush>(type, shaderStage));
      }


    private:
      vt m_type;
      ShaderStageFlags m_shaderStage;

    public:
      inline fun type() { return m_type; }
      inline fun shaderStage() { return m_shaderStage; }
  };



  struct slotPush
  {
    public:
      slotPush(info<slotPush> *info, rhi::buffer *buf)
        : m_info(info)
        , m_buf(buf)
      {
        assert(info && buf);
      }


    private:
      info<slotPush> *m_info;
      rhi::buffer *m_buf;
    
    public:
      inline fun info() { return m_info; }
      inline fun buf() { return m_buf; }
  };

}
