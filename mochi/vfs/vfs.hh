/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include <cstddef>
#include <iostream>
#include <istream>
#include <string_view>



namespace mochi::vfs
{

	struct Mapped {
		protected:
			Mapped() = default;

		public:
			virtual ~Mapped() = default;

		protected:
			void* m_data{};
			usize m_size;

		public:
			fun data() const { return m_data; }
			fun size() const { return m_size; }

			fun view() const { return std::string_view{(char*)data(), size()}; }
			fun span() const { return std::span<char>{(char*)data(), size()}; }

		public:
			operator ::data() { return {data(), size()}; }
	};


	/// @brief Virtual file system provider interface
	struct Provider {
		protected:
			Provider() = default;
		
		public:
			virtual ~Provider() = default;

		public:
			fun (*exists)(Provider*, std::string_view fpath) -> bool = {};
		
			fun (*open_ro)(Provider*, std::string_view fpath) -> sptr<std::istream> = {};
			fun (*open_map)(Provider*, std::string_view fpath) -> sptr<Mapped> = {};
			fun (*open_rw)(Provider*, std::string_view fpath) -> sptr<std::iostream> = {};
	};



	/// @brief Registers a VFS provider
	fun provider_reg(std::string protocol, Provider* provider) -> void;
	/// @brief Deletes a VFS provider
	fun provider_del(std::string protocol) -> void;


	fun exists(std::string_view fpath) -> bool;

	/// @brief Open a file path and opens it in read-only mode
	fun open_ro(std::string_view fpath) -> sptr<std::istream>;
	/// @brief Open a file path and opens it in read-only mode
	fun open_map(std::string_view fpath) -> sptr<Mapped>;
	/// @brief Open a file path and opens it in read-write mode
	fun open_rw(std::string_view fpath) -> sptr<std::iostream>;
}



inline fun operator""_vfs_ro (const char *fpath, std::size_t len) -> sptr<std::istream> {
	return mochi::vfs::open_ro(std::string_view(fpath, len));
}

inline fun operator""_vfs_map (const char *fpath, std::size_t len) -> sptr<mochi::vfs::Mapped> {
	return mochi::vfs::open_map(std::string_view(fpath, len));
}

inline fun operator""_vfs_rw (const char *fpath, std::size_t len) -> sptr<std::iostream> {
	return mochi::vfs::open_rw(std::string_view(fpath, len));
}
