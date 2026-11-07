/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};

use crate::manager::device::Device;
use crate::sync::FrameSynchronizer;
use crate::traits::RhiBackend;


#[derive(Clone)]
pub struct Synchronizer<B: RhiBackend> {
	sync: Arc<Mutex<FrameSynchronizer<B>>>,
	device: Arc<B::Device>,
}

unsafe impl<B: RhiBackend> Send for Synchronizer<B> {}
unsafe impl<B: RhiBackend> Sync for Synchronizer<B> {}

impl<B: RhiBackend> Synchronizer<B> {
	pub fn new(device: &Device<B>, max_frames_in_flight: u32) -> Self {
		let frame_sync = FrameSynchronizer::new(device.raw(), max_frames_in_flight as usize)
			.expect("Failed to create FrameSynchronizer");
		Self {
			sync: Arc::new(Mutex::new(frame_sync)),
			device: device.raw_arc().clone(),
		}
	}

	pub fn begin_frame(&mut self) {
		self
			.sync
			.lock()
			.unwrap()
			.begin_frame(&self.device)
			.expect("Failed to wait/reset fence on begin_frame");
	}

	pub fn end_frame(&mut self) {
		self.sync.lock().unwrap().end_frame();
	}

	pub fn current_frame(&self) -> u32 {
		self.sync.lock().unwrap().current_frame() as u32
	}

	pub fn image_available_semaphore(&self) -> B::Semaphore {
		self.sync.lock().unwrap().image_available_semaphore().clone()
	}

	pub fn in_flight_fence(&self) -> B::Fence {
		self.sync.lock().unwrap().in_flight_fence().clone()
	}
}
