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
#include "mochi/rhi/rhi.hh"
#include "mochi/rhi/slotPush.hh"
#include <cassert>
#include <vector>



namespace mochi::rhi
{

  template<>
  struct info<listPush>
  {
    public:
      info(ShaderStageFlags stage, std::vector<info<slotPush>> ipush);


    private:
      ShaderStageFlags m_stage;
      std::vector<info<slotPush>> m_ipush;
      u32 m_size{};
      
    public:
      inline fun  stage() { return m_stage; }
      inline fun& ipush() { return m_ipush; }
      inline fun  size() { return m_size; }
  };




  struct listPush
  {
    public:
      template <typename... T>
      inline listPush(std::tuple<T...> tup) {

        constexpr auto align_size = [](u64 size, u64 alignment) {
          if (alignment == 0) return size; else return ((size + alignment - 1) / alignment) * alignment;
        };

        m_size = sizeof...(T);

        u32 size{};

        std::apply([&](const auto&... args) {(([&]()
        {
          size = align_size(size, alignof(args));

          size += sizeof(args);

        }()), ...);}, tup);


        m_data = {std::malloc(size), size};


        u32 off{};

        std::apply([&](const auto&... args) {(([&]()
        {
          off = align_size(off, alignof(args));

          __builtin_memcpy((u8*)m_data.ptr() +off, &args, sizeof(args));
          m_offs.push_back({off, sizeof(args)});

          off += sizeof(args);

        }()), ...);}, tup);
      }


      listPush(const listPush&) = delete;
      listPush& operator=(const listPush&) = delete;

    private:
      data m_data{};
      std::vector<offs> m_offs;
      u32 m_size{};

    public:
      inline fun  data() { return m_data; }
      inline fun& offs() { return m_offs; }
      inline fun  size() { return m_size; }

    public:
      inline operator std::vector<::data>() {
        std::vector<::data> ret;
        ret.reserve(m_offs.size());

        for (auto &x: m_offs)
          ret.push_back({(u8*)m_data.ptr()+x.off(), x.size()});

        return ret;
      }

  };

}
