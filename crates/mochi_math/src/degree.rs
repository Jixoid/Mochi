/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::Float;


#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Rad<T: Float = f32> {
  pub rad: T
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Deg<T: Float = f32> {
  pub deg: T
}


impl<T: Float> Rad<T> {
  pub const ZERO: Self = Self{rad: T::ZERO};

  pub const fn new(rad: T) -> Self {
    Self { rad }
  }

  pub fn to_degrees(self) -> Deg<T> {
    Deg{deg: self.rad.to_degrees()}
  }

  pub fn sin(self) -> T {
    self.rad.sin()
  }
  
  pub fn cos(self) -> T {
    self.rad.cos()
  }
  
  pub fn tan(self) -> T {
    self.rad.tan()
  }
  
  pub fn sin_cos(self) -> (T, T) {
    self.rad.sin_cos()
  }

  pub fn asin(val: T) -> Self {
    Self { rad: val.asin() }
  }

  pub fn acos(val: T) -> Self {
    Self { rad: val.acos() }
  }

  pub fn atan2(y: T, x: T) -> Self {
    Self { rad: y.atan2(x) }
  }
}

impl<T: Float> Deg<T> {
  pub const ZERO: Self = Self{deg: T::ZERO};

  pub const fn new(deg: T) -> Self {
    Self { deg }
  }

  pub fn to_radians(self) -> Rad<T> {
    Rad {rad: self.deg.to_radians()}
  }
}


impl<T: Float> From<Deg<T>> for Rad<T> {
  fn from(d: Deg<T>) -> Self {
    d.to_radians()
  }
}

impl<T: Float> From<Rad<T>> for Deg<T> {
  fn from(r: Rad<T>) -> Self {
    r.to_degrees()
  }
}


impl<T: Float> Neg for Rad<T> {
  type Output = Self;
  
  fn neg(self) -> Self {
    Self{rad: -self.rad}
  }
}

impl<T: Float> Neg for Deg<T> {
  type Output = Self;
  
  fn neg(self) -> Self {
    Self{deg: -self.deg}
  }
}


impl<T: Float> Add<Self> for Rad<T> {
  type Output = Self;

  fn add(self, rhs: Self) -> Self {
    Self { rad: self.rad + rhs.rad }
  }
}

impl<T: Float> Sub<Self> for Rad<T> {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self {
    Self { rad: self.rad - rhs.rad }
  }
}


impl<T: Float> Add<Self> for Deg<T> {
  type Output = Self;

  fn add(self, rhs: Self) -> Self {
    Self { deg: self.deg + rhs.deg }
  }
}

impl<T: Float> Sub<Self> for Deg<T> {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self {
    Self { deg: self.deg - rhs.deg }
  }
}


impl<T: Float> Mul<T> for Rad<T> {
  type Output = Self;

  fn mul(self, rhs: T) -> Self {
    Self { rad: self.rad * rhs }
  }
}

impl<T: Float> Div<T> for Rad<T> {
  type Output = Self;

  fn div(self, rhs: T) -> Self {
    Self { rad: self.rad / rhs }
  }
}

impl<T: Float> Div<Self> for Rad<T> {
  type Output = T;

  fn div(self, rhs: Self) -> T {
    self.rad / rhs.rad
  }
}


impl<T: Float> Mul<T> for Deg<T> {
  type Output = Self;

  fn mul(self, rhs: T) -> Self {
    Self { deg: self.deg * rhs }
  }
}

impl<T: Float> Div<T> for Deg<T> {
  type Output = Self;

  fn div(self, rhs: T) -> Self {
    Self { deg: self.deg / rhs }
  }
}

impl<T: Float> Div<Self> for Deg<T> {
  type Output = T;

  fn div(self, rhs: Self) -> T {
    self.deg / rhs.deg
  }
}
