/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub mod core;
pub mod display;
pub mod rendering;

pub use mochi_ahi as ahi;
pub use mochi_math as math;
pub use mochi_rhi as rhi;

use core::Engine;


fn main() {
	let mut engine = Engine::new("Mochi Engine", 1280, 720);

	engine.set_idle(|_dt| {
		// Frame update
	});

	if let Err(err) = engine.run() {
		eprintln!("[Engine] Runtime error: {err}");
	}
}
