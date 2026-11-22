/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::body::BodyHandle;
use crate::collider::ColliderHandle;
use crate::joint::JointHandle;


#[derive(Clone, Debug, PartialEq)]
pub enum PhiError {
	BodyNotFound(BodyHandle),
	ColliderNotFound(ColliderHandle),
	JointNotFound(JointHandle),
	BackendInitFailed(String),
	InvalidShape(String),
	SimulationError(String),
	UnsupportedFeature(String),
}

impl std::fmt::Display for PhiError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::BodyNotFound(h) => write!(f, "Rigid body handle {:?} not found in physics world", h),
			Self::ColliderNotFound(h) => write!(f, "Collider handle {:?} not found in physics world", h),
			Self::JointNotFound(h) => write!(f, "Joint handle {:?} not found in physics world", h),
			Self::BackendInitFailed(msg) => write!(f, "Physics backend initialization failed: {msg}"),
			Self::InvalidShape(msg) => write!(f, "Invalid collision shape: {msg}"),
			Self::SimulationError(msg) => write!(f, "Physics simulation step error: {msg}"),
			Self::UnsupportedFeature(msg) => write!(f, "Unsupported physics feature: {msg}"),
		}
	}
}

impl std::error::Error for PhiError {}

pub type PhiResult<T> = Result<T, PhiError>;
