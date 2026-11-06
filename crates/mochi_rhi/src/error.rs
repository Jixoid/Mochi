/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::fmt;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RhiError {
	OutOfMemory,
	DeviceLost,
	InitializationFailed(String),
	ResourceCreation(String),
	ShaderCompilation(String),
	PipelineCreation(String),
	SurfaceCreation(String),
	SwapchainOutOfDate,
	SwapchainLost,
	Timeout,
	AlreadyMapped,
	NotMapped,
	Unsupported(String),
	BackendError(String),
}

impl fmt::Display for RhiError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			RhiError::OutOfMemory => write!(f, "RHI: Out of memory"),
			RhiError::DeviceLost => write!(f, "RHI: Device lost"),
			RhiError::InitializationFailed(msg) => write!(f, "RHI initialization failed: {msg}"),
			RhiError::ResourceCreation(msg) => write!(f, "RHI resource creation failed: {msg}"),
			RhiError::ShaderCompilation(msg) => write!(f, "RHI shader compilation failed: {msg}"),
			RhiError::PipelineCreation(msg) => write!(f, "RHI pipeline creation failed: {msg}"),
			RhiError::SurfaceCreation(msg) => write!(f, "RHI surface creation failed: {msg}"),
			RhiError::SwapchainOutOfDate => write!(f, "RHI swapchain out of date (needs resize)"),
			RhiError::SwapchainLost => write!(f, "RHI swapchain surface lost"),
			RhiError::Timeout => write!(f, "RHI operation timed out"),
			RhiError::AlreadyMapped => write!(f, "RHI buffer is already mapped"),
			RhiError::NotMapped => write!(f, "RHI buffer is not mapped"),
			RhiError::Unsupported(msg) => write!(f, "RHI feature unsupported: {msg}"),
			RhiError::BackendError(msg) => write!(f, "RHI backend error: {msg}"),
		}
	}
}

impl std::error::Error for RhiError {}

pub type RhiResult<T> = Result<T, RhiError>;
