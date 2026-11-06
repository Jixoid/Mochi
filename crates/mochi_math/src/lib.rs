/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/

#![feature(const_trait_impl)]
#![feature(const_ops)]
#![feature(f16)]
#![feature(f128)]


mod extent;
mod traits;
mod vec2;
mod vec3;
mod vec4;

pub use extent::*;
pub use traits::*;
pub use vec2::*;
pub use vec3::*;
pub use vec4::*;
