/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include <fstream>
#include <iostream>
#include <istream>
#include <string>
#include <string_view>
#include "mochi/basis.hh"
#include "mochi/vfs/vfs.hh"



namespace vfs::__file
{

	fun get() -> vfs_raii
	{
		return vfs_raii("file", new vfs_provider{
			[]() -> void* {return nil; },
			[](void* _Data) {},

			[](void*, std::string_view fpath) -> sptr<std::istream>
			{
				return make_sptr(new std::ifstream(std::string(fpath)));
			},
			[](void*, std::string_view fpath) -> sptr<std::iostream>
			{
				return make_sptr(new std::fstream(std::string(fpath)));
			}
		});
	}

}
