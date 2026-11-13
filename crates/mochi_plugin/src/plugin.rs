/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use core::ffi;
use bitflags::bitflags;


pub const MOCHI_PLUGIN_VER: u64 = 0x1;

#[repr(u64)]
pub enum HookThing {
	Update,
}

bitflags! {
	#[derive(Copy, Clone)]
	pub struct Ability: u64 {}
}


#[repr(C)]
pub struct PluginContext {
	pub plg_id: u32,
	pub hook: unsafe extern "C" fn(thing: HookThing, hook: unsafe extern "C" fn()),
}

pub const MOCHI_PLUGIN_GET: &str = "MochiPluginGet";
pub type MochiPluginGet = unsafe extern "C" fn() -> *const MochiPlugin;


#[repr(C)]
#[derive(Copy, Clone)]
pub struct MochiPlugin {
	pub version: u64,
	pub name: *const ffi::c_char,
	pub ability: Ability,

	pub on_load: unsafe extern "C" fn(pctx: &PluginContext),
	pub on_unload: unsafe extern "C" fn(pctx: &PluginContext),
}
