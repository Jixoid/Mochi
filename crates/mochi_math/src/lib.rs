/*
	This file is part of QAOS

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
mod degree;
mod traits;
mod vec2;
mod vec3;
mod vec4;
mod quat;
mod mat3x4;
mod mat4x4;
mod affine;
mod transform;
mod euler;
mod aabb;
mod plane;
mod ray;
mod rect;
mod frustum;
mod color;

pub use extent::*;
pub use degree::*;
pub use traits::*;
pub use vec2::*;
pub use vec3::*;
pub use vec4::*;
pub use quat::*;
pub use mat3x4::*;
pub use mat4x4::*;
pub use affine::*;
pub use transform::*;
pub use euler::*;
pub use aabb::*;
pub use plane::*;
pub use ray::*;
pub use rect::*;
pub use frustum::*;
pub use color::*;
