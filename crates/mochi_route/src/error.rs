/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use core::fmt;
use std::ops::BitOr;


#[derive(Debug)]
pub enum Error {
	New,
	Io(std::io::Error),
	Str(String),
}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Self {
		Error::Io(err)
	}
}

impl From<String> for Error {
	fn from(err: String) -> Self {
		Error::Str(err)
	}
}

impl From<&str> for Error {
	fn from(err: &str) -> Self {
		Error::Str(err.to_string())
	}
}

impl BitOr<String> for Error {
	type Output = Error;

	fn bitor(self, rhs: String) -> Self::Output {
		Error::Str(rhs)
	}
}

impl BitOr<&str> for Error {
	type Output = Error;

	fn bitor(self, rhs: &str) -> Self::Output {
		Error::Str(rhs.to_string())
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Error::New => write!(f, "unknown error"),
			Error::Io(e) => write!(f, "io: {}", e),
			Error::Str(v) => write!(f, "{}", v),
		}
	}
}
