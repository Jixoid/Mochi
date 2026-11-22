/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/

//! # Mochi Physics Hardware/Backend Interface (PHI)
//!
//! Abstract, backend-agnostic 3D physics interfaces, components, and math abstractions for the Mochi Engine.
//! Designed to support multiple pluggable backends such as Rapier3D.

pub mod body;
pub mod collider;
pub mod error;
pub mod joint;
pub mod ray;
pub mod traits;
pub mod world;

pub use body::*;
pub use collider::*;
pub use error::*;
pub use joint::*;
pub use ray::*;
pub use traits::*;
pub use world::*;

