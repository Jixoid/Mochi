/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ops::{Mul, Neg};

use crate::{Float, Rad, Vec3};


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Quat<T: Float = f32> {
  pub x: T,
  pub y: T,
  pub z: T,
  pub w: T,
}

impl<T: Float> Quat<T> {
  pub const IDENTITY: Self = Self {x: T::ZERO, y: T::ZERO, z: T::ZERO, w: T::ONE};

  pub const fn new(x: T, y: T, z: T, w: T) -> Self {
    Self { x, y, z, w }
  }

  
  pub fn from_axis_angle(axis: Vec3<T>, angle: Rad<T>) -> Self {
    let axis = axis.normalize();
    let (s, c) = (angle * T::HALF).sin_cos();
    Self::new(axis.x * s, axis.y * s, axis.z * s, c)
  }

  pub fn from_rotation_x(angle: Rad<T>) -> Self {
    let (s, c) = (angle * T::HALF).sin_cos();
    Self::new(s, T::ZERO, T::ZERO, c)
  }

  pub fn from_rotation_y(angle: Rad<T>) -> Self {
    let (s, c) = (angle * T::HALF).sin_cos();
    Self::new(T::ZERO, s, T::ZERO, c)
  }

  pub fn from_rotation_z(angle: Rad<T>) -> Self {
    let (s, c) = (angle * T::HALF).sin_cos();
    Self::new(T::ZERO, T::ZERO, s, c)
  }

  pub fn from_euler_yxz(yaw: Rad<T>, pitch: Rad<T>, roll: Rad<T>) -> Self {
    Self::from_rotation_y(yaw)
      * Self::from_rotation_x(pitch)
      * Self::from_rotation_z(roll)
  }

  pub fn from_two_vectors(u: Vec3<T>, v: Vec3<T>) -> Self {
    let u = u.normalize();
    let v = v.normalize();
    let cos_theta = u.dot(&v);
    let eps = T::from_f32(1e-6);
    if cos_theta < -T::ONE + eps {
      let mut rotation_axis = Vec3::UP.cross(&u);
      if rotation_axis.length() < T::from_f32(0.01) {
        rotation_axis = Vec3::RIGHT.cross(&u);
      }
      return Self::from_axis_angle(rotation_axis.normalize(), Rad::new(T::PI));
    }
    let rotation_axis = u.cross(&v);
    let s = ((T::ONE + cos_theta) * T::TWO).sqrt();
    let inv_s = T::ONE / s;
    Self::new(
      rotation_axis.x * inv_s,
      rotation_axis.y * inv_s,
      rotation_axis.z * inv_s,
      s * T::HALF,
    ).normalize()
  }

  pub fn look_at(pos: Vec3<T>, target: Vec3<T>, up: Vec3<T>) -> Self {
    let forward = (target - pos).normalize();
    let right = forward.cross(&up).normalize();
    let ortho_up = right.cross(&forward).normalize();
    let q1 = Self::from_two_vectors(Vec3::FORWARD, forward);
    let current_up = q1.rotate(Vec3::UP);
    let q2 = Self::from_two_vectors(current_up, ortho_up);
    q2 * q1
  }


  pub fn dot(self, o: Self) -> T {
    self.x * o.x + self.y * o.y + self.z * o.z + self.w * o.w
  }

  pub fn length_squared(self) -> T {
    self.dot(self)
  }

  pub fn length(self) -> T {
    self.length_squared().sqrt()
  }

  pub fn normalize(self) -> Self {
    let len_sq = self.length_squared();
    if len_sq > T::MIN_POSITIVE {
      let inv = T::ONE / len_sq.sqrt();
      Self::new(self.x * inv, self.y * inv, self.z * inv, self.w * inv)
    } else {
      Self::IDENTITY
    }
  }

  pub fn is_normalized(self, eps: T) -> bool {
    (self.length_squared() - T::ONE).abs() <= eps
  }

  pub fn conjugate(self) -> Self {
    Self::new(-self.x, -self.y, -self.z, self.w)
  }

  pub fn inverse(self) -> Self {
    let inv = T::ONE / self.length_squared();
    let c = self.conjugate();
    Self::new(c.x * inv, c.y * inv, c.z * inv, c.w * inv)
  }

  pub fn rotate(self, v: Vec3<T>) -> Vec3<T> {
    // t = 2 * (u x v);  v' = v + w*t + u x t
    let (ux, uy, uz) = (self.x, self.y, self.z);
    let two = T::TWO;
    let tx = two * (uy * v.z - uz * v.y);
    let ty = two * (uz * v.x - ux * v.z);
    let tz = two * (ux * v.y - uy * v.x);
    Vec3::new_with(
      v.x + self.w * tx + (uy * tz - uz * ty),
      v.y + self.w * ty + (uz * tx - ux * tz),
      v.z + self.w * tz + (ux * ty - uy * tx),
    )
  }

  pub fn nlerp(self, mut to: Self, t: T) -> Self {
    if self.dot(to) < T::ZERO {
      to = -to; // en kısa yol
    }
    Self::new(
      self.x + (to.x - self.x) * t,
      self.y + (to.y - self.y) * t,
      self.z + (to.z - self.z) * t,
      self.w + (to.w - self.w) * t,
    )
    .normalize()
  }

  pub fn slerp(self, mut to: Self, t: T) -> Self {
    let mut d = self.dot(to);
    if d < T::ZERO {
      to = -to;
      d = -d;
    }
    if d > T::from_f32(0.9995) {
      return self.nlerp(to, t);
    }
    let theta = d.acos();
    let inv_sin = T::ONE / theta.sin();
    let a = ((T::ONE - t) * theta).sin() * inv_sin;
    let b = (t * theta).sin() * inv_sin;
    Self::new(
      self.x * a + to.x * b,
      self.y * a + to.y * b,
      self.z * a + to.z * b,
      self.w * a + to.w * b,
    )
  }

  pub fn to_axis_angle(self) -> (Vec3<T>, T) {
    let q = if self.w < T::ZERO { -self } else { self };
    let w = q.w.clamp(-T::ONE, T::ONE);
    let s = (T::ONE - w * w).sqrt();
    if s < T::from_f32(1e-6) {
      (Vec3::new_with(T::ONE, T::ZERO, T::ZERO), T::ZERO)
    } else {
      (Vec3::new_with(q.x / s, q.y / s, q.z / s), T::TWO * w.acos())
    }
  }

}


impl<T: Float> Default for Quat<T> {
  fn default() -> Self {
    Self::IDENTITY
  }
}


impl<T: Float> Neg for Quat<T> {
  type Output = Self;
  
  fn neg(self) -> Self {
    Self::new(-self.x, -self.y, -self.z, -self.w)
  }
}


impl<T: Float> Mul<Self> for Quat<T> {
  type Output = Self;
  
  fn mul(self, r: Self) -> Self {
    Self::new(
      self.w * r.x + self.x * r.w + self.y * r.z - self.z * r.y,
      self.w * r.y - self.x * r.z + self.y * r.w + self.z * r.x,
      self.w * r.z + self.x * r.y - self.y * r.x + self.z * r.w,
      self.w * r.w - self.x * r.x - self.y * r.y - self.z * r.z,
    )
  }
}

impl<T: Float> Mul<Vec3<T>> for Quat<T> {
  type Output = Vec3<T>;

  fn mul(self, v: Vec3<T>) -> Vec3<T> {
    self.rotate(v)
  }
}
