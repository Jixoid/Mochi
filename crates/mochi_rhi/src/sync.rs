/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::error::RhiResult;
use crate::traits::{RhiBackend, RhiDevice};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Semaphore(pub *mut std::ffi::c_void);
unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fence(pub *mut std::ffi::c_void);
unsafe impl Send for Fence {}
unsafe impl Sync for Fence {}


pub struct FrameSynchronizer<B: RhiBackend> {
	image_available_semaphores: Vec<B::Semaphore>,
	in_flight_fences: Vec<B::Fence>,
	current_frame: usize,
	max_frames_in_flight: usize,
}

impl<B: RhiBackend> FrameSynchronizer<B> {
	pub fn new(device: &B::Device, max_frames_in_flight: usize) -> RhiResult<Self> {
		let mut image_available_semaphores = Vec::with_capacity(max_frames_in_flight);
		let mut in_flight_fences = Vec::with_capacity(max_frames_in_flight);

		for _ in 0..max_frames_in_flight {
			image_available_semaphores.push(device.create_semaphore()?);
			in_flight_fences.push(device.create_fence(true)?);
		}

		Ok(Self {
			image_available_semaphores,
			in_flight_fences,
			current_frame: 0,
			max_frames_in_flight,
		})
	}

	
	pub fn begin_frame(&mut self, device: &B::Device) -> RhiResult<()> {
		let fence = &self.in_flight_fences[self.current_frame];
		device.wait_fence(fence, u64::MAX)?;
		device.reset_fence(fence)?;
		Ok(())
	}

	pub fn end_frame(&mut self) {
		self.current_frame = (self.current_frame + 1) % self.max_frames_in_flight;
	}

	pub fn current_frame(&self) -> usize {
		self.current_frame
	}

	pub fn max_frames_in_flight(&self) -> usize {
		self.max_frames_in_flight
	}

	pub fn image_available_semaphore(&self) -> &B::Semaphore {
		&self.image_available_semaphores[self.current_frame]
	}

	pub fn in_flight_fence(&self) -> &B::Fence {
		&self.in_flight_fences[self.current_frame]
	}
}
