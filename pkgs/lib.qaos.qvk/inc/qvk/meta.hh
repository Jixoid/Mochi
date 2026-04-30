/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.h"
#include "Basis.hh"
#include "qvk/device.hh"
#include "qvk/scene.hh"
#include "qvk/types.hh"
#include <algorithm>
#include <cassert>
#include <concepts>
#include <unordered_map>
#include <vector>
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>




namespace
{
  enum metaEnum: u8 { meScene, meCamera };
}

namespace qvk
{
  template <typename _id, typename _payload>
  struct metapay
  {
    public:
      inline metapay(_id id, _payload payload)
        : m_id(id)
        , m_payload(payload)
      {}
    

    private:
      _id m_id;
      _payload m_payload;

    public:
      inline fun operator->() -> _payload { return m_payload; }

      inline operator _payload() { return m_payload; }

  };



  template <typename T>
  struct metaobj;
  

  template <>
  struct metaobj<scene>
  {
    friend struct meta;

    private:
      scene *m_self{};
      std::vector<metaobj<camera>*> m_cameras;

    public:
      inline fun self() { return m_self; }
      
      inline fun add(metaobj<camera> *obj) {
        
        assert(std::find(m_cameras.begin(), m_cameras.end(), obj) == m_cameras.end() && "object already owned");

        m_cameras.push_back(obj);
      }

      inline fun connect(scene *obj) {
        assert(!m_self && "already has parent");

        m_self = obj;
      }

  };

  template <>
  struct metaobj<camera>
  {
    friend struct meta;

    private:
      camera *m_self{};

    public:
      inline fun self() { return m_self; }

      inline fun connect(camera *obj) {
        assert(!m_self && "already has parent");

        m_self = obj;
      }

  };

  template <>
  struct metaobj<pipeline>
  {
    friend struct meta;

    private:
      pipeline *m_self{};

    public:
      inline fun self() { return m_self; }

      inline fun connect(pipeline *obj) {
        assert(!m_self && "already has parent");

        m_self = obj;
      }

  };




  struct meta
  {
    public:
      explicit meta(qvk::device &device);

      inline ~meta() {
        for (auto &X: m_list) del(metaEnum(X.first >> (INTPTR_WIDTH-8-1)), X.second);
      }


    private:
      device &m_device;
      metaid m_lastid :(INTPTR_WIDTH-8) {1};
      std::unordered_map<metaid, void*> m_list;


    private:
      template <metaEnum E>
      inline fun newid() -> metaid {
        assert(m_lastid+1 && "metaid reseted");
        return ((metaid)E << (INTPTR_WIDTH-8-1)) | (m_lastid++);
      }

      inline fun del(metaEnum kind, void* obj) -> void { switch (kind)
      {
        case meScene: delete (metaobj<scene>*)obj; return;
        case meCamera: delete (metaobj<camera>*)obj; return;
      }}
      


    public:
      inline fun del(metaid id) -> void {
        auto it = m_list.find(id);

        #ifdef _QVK_debug_ownership
        if (m_list.end() == it)
          assert(false && "illegal metaid");
        #endif

        del(metaEnum(id >> (INTPTR_WIDTH-8-1)), it->second);
      }

    

    public:
      template <typename T>
        requires std::derived_from<T, qvk::scene>
      inline fun make() -> metapay<metaid, metaobj<scene>*> {
        auto id = newid<meScene>();
        auto mo = new metaobj<scene>{};

        m_list[id] = mo;

        return {id, mo};
      }


      template <typename T>
        requires std::derived_from<T, qvk::camera>
      inline fun make() -> metapay<metaid, metaobj<camera>*> {
        auto id = newid<meCamera>();
        auto mo = new metaobj<camera>{};

        m_list[id] = mo;

        return {id, mo};
      }


  };

}
