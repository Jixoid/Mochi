/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/
#pragma once

#include "mochi/basis.hh"
#include <cassert>
#include <format>
#include <type_traits>
#include <utility>
#include <algorithm>



namespace mochi
{

  template <usize S, typename T>
  struct extent
  {
    public:
      template <typename... Args>
      constexpr extent(Args... args): data{static_cast<T>(args)...} {}

    protected:
      std::array<T,S> data;

    public:
      constexpr fun operator==(const extent&) const -> bool = default;
      constexpr fun operator!=(const extent&) const -> bool = default;

    public:
      constexpr fun operator[](usize idx) -> T& {
        assert(idx < S && "index out of bounds for extent.");
        return data[idx];
      }
      
      constexpr fun operator[](usize idx) const -> const T& {
        assert(idx < S && "index out of bounds for extent.");
        return data[idx];
      }

    public:
      constexpr fun x() -> T& requires (S >= 1) { return data[0]; }
      constexpr fun y() -> T& requires (S >= 2) { return data[1]; }
      constexpr fun z() -> T& requires (S >= 3) { return data[2]; }
      constexpr fun w() -> T& requires (S >= 4) { return data[3]; }

      constexpr fun x() const -> T& requires (S >= 1) { return data[0]; }
      constexpr fun y() const -> T& requires (S >= 2) { return data[1]; }
      constexpr fun z() const -> T& requires (S >= 3) { return data[2]; }
      constexpr fun w() const -> T& requires (S >= 4) { return data[3]; }
  };

  template <typename... Args>
  extent(Args...) -> extent<sizeof...(Args), std::common_type_t<Args...>>;




  template <std::size_t i, typename T, std::size_t S>
  [[nodiscard]] inline constexpr auto& get(extent<S,T> &v) noexcept {
    static_assert(i < S, "Index out of bounds!");
    return v.data[i];
  }

  template <std::size_t i, typename T, std::size_t S>
  [[nodiscard]] inline constexpr const auto& get(const extent<S,T> &v) noexcept {
    static_assert(i < S, "Index out of bounds!");
    return v.data[i];
  }

  template <std::size_t i, typename T, std::size_t S>
  [[nodiscard]] inline constexpr auto&& get(extent<S,T> &&v) noexcept {
    static_assert(i < S, "Index out of bounds!");
    return std::move(v.data[i]);
  }

  template <std::size_t i, typename T, std::size_t S>
  [[nodiscard]] inline constexpr const auto&& get(const extent<S,T> &&v) noexcept {
    static_assert(i < S, "Index out of bounds!");
    return std::move(v.data[i]);
  }

}


namespace std {
  template <typename T, std::size_t S>
  struct tuple_size<mochi::extent<S,T>>: std::integral_constant<std::size_t, S> {};

  template <std::size_t i, typename T, std::size_t S>
  struct tuple_element<i, mochi::extent<S,T>> {
    static_assert(i < S, "index out of bounds for extent");
    using type = T;
  };

  template <typename T, std::size_t S>
  struct formatter<mochi::extent<S,T>> {
    constexpr auto parse(std::format_parse_context &ctx) { return ctx.begin(); }

    auto format(const mochi::extent<S,T> &ext, std::format_context &ctx) const {
      auto out = ctx.out();
      
      out = std::format_to(out, "extent<{}, {}>[", typeid(T).name(), S);
      for (std::size_t i = 0; i < S; i++) {
        out = std::format_to(out, "{}", ext.data[i]);
        if (i+1 < S) {
          out = std::format_to(out, ", ");
        }
      }
      return std::format_to(out, "]");
    }
  };
}


template <typename T, std::size_t S>
fun operator<<(std::ostream &os, const mochi::extent<S,T> &ext) -> std::ostream& {
  os << "{";
  for (std::size_t i = 0; i < S; i++) {
    os << ext.data[i];
    if (i+1 < S)
      os << ", ";
  }
  os << "}";
  
  return os;
}
