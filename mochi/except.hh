/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include <exception>
#include <string>



namespace mochi
{

  /** @brief Custom exception class for Mochi engine errors. */
  class except: public std::exception
  {
    public:
      /**
       * @brief Construct a new exception.
       * @param str The error message.
       */
      explicit except(const std::string &str): m_err(str) {}
    
    
    private:
      std::string m_err{};

    public:
      /** @brief Access the error message string. */
      inline fun& err() { return m_err; }


    public:
      /**
       * @brief Get the C-string representation of the error message.
       * @return const char* Pointer to the error message.
       */
      inline fun what() const noexcept -> const char* override { return m_err.c_str(); }
  };


  
  /** @brief Custom exception class for Mochi engine IO errors. */
  class io_error: public except
  {
    public:
      explicit io_error(const std::string &str): except(str) {}
  };

  /** @brief Custom exception class for Mochi engine RHI/Rendering errors. */
  class rhi_error: public except
  {
    public:
      explicit rhi_error(const std::string &str): except(str) {}
  };

  /** @brief Custom exception class for Mochi engine Asset/Parsing errors. */
  class asset_error: public except
  {
    public:
      explicit asset_error(const std::string &str): except(str) {}
  };

}
