/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};

use crate::command::Command;
use crate::manager::device::Device;
use crate::traits::{RhiBackend, RhiCommandQueue};


#[derive(Clone)]
pub struct CommandManager<B: RhiBackend> {
	inner: Arc<B::CommandQueue>,
	submit_lock: Arc<Mutex<()>>,
}

unsafe impl<B: RhiBackend> Send for CommandManager<B> {}
unsafe impl<B: RhiBackend> Sync for CommandManager<B> {}

impl<B: RhiBackend> CommandManager<B> {
	pub fn new(device: &Device<B>) -> Self {
		let queue = B::create_command_queue(device.raw())
			.expect("Failed to initialize RHI command queue");
		Self {
			inner: Arc::new(queue),
			submit_lock: Arc::new(Mutex::new(())),
		}
	}

	pub fn raw(&self) -> &B::CommandQueue {
		&self.inner
	}

	pub fn allocate_graphic_command(&self) -> Result<Command<B>, String> {
		let cmd = self
			.inner
			.allocate_graphic_command()
			.map_err(|e| format!("Failed to allocate graphic command: {e}"))?;
		Ok(Command::from_raw(cmd))
	}

	pub fn submit(&self, cmd: &Command<B>) {
		let _lock = self.submit_lock.lock().unwrap();
		self.inner.submit(cmd.raw()).expect("Command submit failed");
	}

	pub fn submit_with_sync(
		&self,
		cmd: &Command<B>,
		wait_semaphore: Option<&B::Semaphore>,
		signal_semaphore: Option<&B::Semaphore>,
		fence: Option<&B::Fence>,
	) {
		let _lock = self.submit_lock.lock().unwrap();
		self
			.inner
			.submit_with_sync(cmd.raw(), wait_semaphore, signal_semaphore, fence)
			.expect("Command submit_with_sync failed");
	}
}
