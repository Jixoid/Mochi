/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "mochi/vfs/vfs.hh"
#include <filesystem>
#include <istream>
#include <string>
#include <string_view>
#include <unordered_map>
#include <utility>



namespace mochi::vfs
{
	std::unordered_map<std::string, Provider*> providers __attribute__((init_priority(65534)));
	std::shared_mutex providers_mtx __attribute__((init_priority(65534)));

	

  fun provider_reg(std::string protocol, Provider *provider) -> void {
    ulock lock(providers_mtx);
    providers[protocol] = provider;
  }

	fun provider_del(std::string protocol) -> void {
		ulock lock(providers_mtx);
    if (auto X = providers.find(protocol); X != providers.end()) {
      delete X->second;
      providers.erase(X);
    }
  }



	fun get_coor(std::string_view fpath) {
		std::string_view payload;
		std::string protocol;

		auto splitPos = fpath.find("://", 1);


		if (splitPos != std::string_view::npos) [[likely]] {
			protocol = fpath.substr(0, splitPos);
			payload  = fpath.substr(splitPos+3);
		}
		else [[unlikely]] {
			protocol = "file";
			payload  = fpath;
		}

		return std::pair{protocol, payload};
	}




	fun exists(std::string_view fpath) -> bool {
		auto coor = get_coor(fpath);

		slock lock(providers_mtx);
		if (auto X = vfs::providers.find(coor.first); X != vfs::providers.end()) {
			if (X->second->exists)
				return X->second->exists(X->second, coor.second);
			else
				throw std::filesystem::filesystem_error("Virtual file could not be opened", "exists", std::make_error_code(std::errc::protocol_not_supported));
		}
		else
			throw std::filesystem::filesystem_error("Virtual file could not be opened", coor.first, std::make_error_code(std::errc::protocol_not_supported));
	}


  fun open_ro(std::string_view fpath) -> sptr<std::istream> {
		auto coor = get_coor(fpath);

		slock lock(providers_mtx);
		if (auto X = vfs::providers.find(coor.first); X != vfs::providers.end()) {
			if (X->second->open_ro)
				return X->second->open_ro(X->second, coor.second);
			else
				throw std::filesystem::filesystem_error("Virtual file could not be opened", "open_ro", std::make_error_code(std::errc::protocol_not_supported));
		}
		else
			throw std::filesystem::filesystem_error("Virtual file could not be opened", coor.first, std::make_error_code(std::errc::protocol_not_supported));
	}

	fun open_map(std::string_view fpath) -> sptr<Mapped> {
		auto coor = get_coor(fpath);


		slock lock(providers_mtx);
		if (auto X = vfs::providers.find(coor.first); X != vfs::providers.end()) {
			if (X->second->open_map)
				return X->second->open_map(X->second, coor.second);
			else
				throw std::filesystem::filesystem_error("Virtual file could not be opened", "open_map", std::make_error_code(std::errc::protocol_not_supported));
		}
		else
			throw std::filesystem::filesystem_error("Virtual file could not be opened", coor.first, std::make_error_code(std::errc::protocol_not_supported));
	}

  fun open_rw(std::string_view fpath) -> sptr<std::iostream> {
		auto coor = get_coor(fpath);

		slock lock(providers_mtx);
		if (auto X = vfs::providers.find(coor.first); X != vfs::providers.end()) {
			if (X->second->open_rw)
				return X->second->open_rw(X->second, coor.second);
			else
				throw std::filesystem::filesystem_error("Virtual file could not be opened", "open_rw", std::make_error_code(std::errc::protocol_not_supported));
		}
		else
			throw std::filesystem::filesystem_error("Virtual file could not be opened", coor.first, std::make_error_code(std::errc::protocol_not_supported));
	}

}
