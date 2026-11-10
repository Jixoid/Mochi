/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::fmt;


#[derive(Debug, Clone)]
pub enum AhiError {
	DeviceInitializationFailed(String),
	DeviceNotAvailable(String),
	StreamError(String),
	DecodeError(String),
	UnsupportedFormat(String),
	IoError(String),
}

impl fmt::Display for AhiError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::DeviceInitializationFailed(msg) => write!(f, "Audio device initialization failed: {msg}"),
			Self::DeviceNotAvailable(msg) => write!(f, "Audio device not available: {msg}"),
			Self::StreamError(msg) => write!(f, "Audio stream error: {msg}"),
			Self::DecodeError(msg) => write!(f, "Audio decode error: {msg}"),
			Self::UnsupportedFormat(msg) => write!(f, "Unsupported audio format: {msg}"),
			Self::IoError(msg) => write!(f, "Audio IO error: {msg}"),
		}
	}
}

impl std::error::Error for AhiError {}

pub type AhiResult<T> = Result<T, AhiError>;
