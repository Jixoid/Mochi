/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ops::{Index, IndexMut};


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Extent<const S: usize, T> {
	pub data: [T; S],
}

impl<const S: usize, T> Extent<S, T> {
	pub const fn new(data: [T; S]) -> Self {
		Self { data }
	}
}


/*
impl<const S: usize, T: Copy> Extent<S, T> {
	pub fn x(&self) -> T {
		self.data[0]
	}

	pub fn y(&self) -> T {
		assert!(S >= 2, "Extent dimension < 2");
		self.data[1]
	}

	pub fn z(&self) -> T {
		assert!(S >= 3, "Extent dimension < 3");
		self.data[2]
	}

	pub fn w(&self) -> T {
		assert!(S >= 4, "Extent dimension < 4");
		self.data[3]
	}
}
*/


impl<const S: usize, T> Index<usize> for Extent<S, T> {
	type Output = T;
	fn index(&self, index: usize) -> &Self::Output {
		&self.data[index]
	}
}

impl<const S: usize, T> IndexMut<usize> for Extent<S, T> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.data[index]
	}
}
