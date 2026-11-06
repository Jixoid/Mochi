/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ops::{Add, Div, Mul, Neg, Sub};


pub const trait Number:
  Copy + Clone +
  Zero + Sqrt + One +
  ConstZero + ConstOne +
  Min + Max + Clamp +
  Abs +
  Add<Self, Output = Self> + Sub<Self, Output = Self> + Mul<Self, Output = Self> + Div<Self, Output = Self> +
  PartialEq + PartialOrd +
{}

const impl Number for f16  {}
const impl Number for f32  {}
const impl Number for f64  {}
const impl Number for f128 {}

const impl Number for i8   {}
const impl Number for i16  {}
const impl Number for i32  {}
const impl Number for i64  {}
const impl Number for i128 {}

const impl Number for u8   {}
const impl Number for u16  {}
const impl Number for u32  {}
const impl Number for u64  {}
const impl Number for u128 {}


pub const trait Float:
  Number +
  Copy + Clone +
  Zero + Sqrt + One +
  ConstZero + ConstOne +
  Pi +
  Min + Max + Clamp +
  const Neg<Output = Self> +
  Add<Self, Output = Self> + Sub<Self, Output = Self> + Mul<Self, Output = Self> + Div<Self, Output = Self> +
  PartialEq + PartialOrd +
{
  const EPSILON: Self;
  const MIN_POSITIVE: Self;
  const HALF: Self;
  const TWO: Self;

  fn from_f32(val: f32) -> Self;
  fn from_f64(val: f64) -> Self;
  fn to_f32(self) -> f32;
  fn to_f64(self) -> f64;
}

const impl Float for f16  {
  const EPSILON: Self = f16::EPSILON;
  const MIN_POSITIVE: Self = f16::MIN_POSITIVE;
  const HALF: Self = 0.5_f16;
  const TWO: Self = 2.0_f16;

  fn from_f32(val: f32) -> Self { val as f16 }
  fn from_f64(val: f64) -> Self { val as f16 }
  fn to_f32(self) -> f32 { self as f32 }
  fn to_f64(self) -> f64 { self as f64 }
}

const impl Float for f32  {
  const EPSILON: Self = f32::EPSILON;
  const MIN_POSITIVE: Self = f32::MIN_POSITIVE;
  const HALF: Self = 0.5_f32;
  const TWO: Self = 2.0_f32;

  fn from_f32(val: f32) -> Self { val }
  fn from_f64(val: f64) -> Self { val as f32 }
  fn to_f32(self) -> f32 { self }
  fn to_f64(self) -> f64 { self as f64 }
}

const impl Float for f64  {
  const EPSILON: Self = f64::EPSILON;
  const MIN_POSITIVE: Self = f64::MIN_POSITIVE;
  const HALF: Self = 0.5_f64;
  const TWO: Self = 2.0_f64;

  fn from_f32(val: f32) -> Self { val as f64 }
  fn from_f64(val: f64) -> Self { val }
  fn to_f32(self) -> f32 { self as f32 }
  fn to_f64(self) -> f64 { self }
}

const impl Float for f128 {
  const EPSILON: Self = f128::EPSILON;
  const MIN_POSITIVE: Self = f128::MIN_POSITIVE;
  const HALF: Self = 0.5_f128;
  const TWO: Self = 2.0_f128;

  fn from_f32(val: f32) -> Self { val as f128 }
  fn from_f64(val: f64) -> Self { val as f128 }
  fn to_f32(self) -> f32 { self as f32 }
  fn to_f64(self) -> f64 { self as f64 }
}


pub trait Pi
where
  Self: Sized
{
  const PI: Self;
  const PI2: Self;
  
  fn to_degrees(self) -> Self;
  fn to_radians(self) -> Self;
  fn sin(self) -> Self;
  fn cos(self) -> Self;
  fn tan(self) -> Self;
  fn sin_cos(self) -> (Self, Self);
  fn asin(self) -> Self;
  fn acos(self) -> Self;
  fn atan2(self, other: Self) -> Self;
}

impl Pi for f16  {
  const PI: Self = core::f16::consts::PI;
  const PI2: Self = core::f16::consts::TAU;

  fn to_degrees(self) -> Self { self.to_degrees() }
  fn to_radians(self) -> Self { self.to_radians() }
  fn sin(self) -> Self { self.sin() }
  fn cos(self) -> Self { self.cos() }
  fn tan(self) -> Self { self.tan() }
  fn sin_cos(self) -> (Self, Self) { self.sin_cos() }
  fn asin(self) -> Self { self.asin() }
  fn acos(self) -> Self { self.acos() }
  fn atan2(self, other: Self) -> Self { self.atan2(other) }
}

impl Pi for f32  {
  const PI: Self = core::f32::consts::PI;
  const PI2: Self = core::f32::consts::TAU;
  
  fn to_degrees(self) -> Self { self.to_degrees() }
  fn to_radians(self) -> Self { self.to_radians() }
  fn sin(self) -> Self { self.sin() }
  fn cos(self) -> Self { self.cos() }
  fn tan(self) -> Self { self.tan() }
  fn sin_cos(self) -> (Self, Self) { self.sin_cos() }
  fn asin(self) -> Self { self.asin() }
  fn acos(self) -> Self { self.acos() }
  fn atan2(self, other: Self) -> Self { self.atan2(other) }
}

impl Pi for f64  {
  const PI: Self = core::f64::consts::PI;
  const PI2: Self = core::f64::consts::TAU;
  
  fn to_degrees(self) -> Self { self.to_degrees() }
  fn to_radians(self) -> Self { self.to_radians() }
  fn sin(self) -> Self { self.sin() }
  fn cos(self) -> Self { self.cos() }
  fn tan(self) -> Self { self.tan() }
  fn sin_cos(self) -> (Self, Self) { self.sin_cos() }
  fn asin(self) -> Self { self.asin() }
  fn acos(self) -> Self { self.acos() }
  fn atan2(self, other: Self) -> Self { self.atan2(other) }
}

impl Pi for f128 {
  const PI: Self = core::f128::consts::PI;
  const PI2: Self = core::f128::consts::TAU;
  
  fn to_degrees(self) -> Self { self.to_degrees() }
  fn to_radians(self) -> Self { self.to_radians() }
  fn sin(self) -> Self { self.sin() }
  fn cos(self) -> Self { self.cos() }
  fn tan(self) -> Self { self.tan() }
  fn sin_cos(self) -> (Self, Self) { self.sin_cos() }
  fn asin(self) -> Self { self.asin() }
  fn acos(self) -> Self { self.acos() }
  fn atan2(self, other: Self) -> Self { self.atan2(other) }
}


pub trait ConstZero {
  const ZERO: Self;
}

impl ConstZero for f16  { const ZERO: Self = 0.0_f16; }
impl ConstZero for f32  { const ZERO: Self = 0.0_f32; }
impl ConstZero for f64  { const ZERO: Self = 0.0_f64; }
impl ConstZero for f128 { const ZERO: Self = 0.0_f128; }

impl ConstZero for i8   { const ZERO: Self = 0_i8; }
impl ConstZero for i16  { const ZERO: Self = 0_i16; }
impl ConstZero for i32  { const ZERO: Self = 0_i32; }
impl ConstZero for i64  { const ZERO: Self = 0_i64; }
impl ConstZero for i128 { const ZERO: Self = 0_i128; }

impl ConstZero for u8   { const ZERO: Self = 0_u8; }
impl ConstZero for u16  { const ZERO: Self = 0_u16; }
impl ConstZero for u32  { const ZERO: Self = 0_u32; }
impl ConstZero for u64  { const ZERO: Self = 0_u64; }
impl ConstZero for u128 { const ZERO: Self = 0_u128; }


pub trait ConstOne {
  const ONE: Self;
}

impl ConstOne for f16  { const ONE: Self = 1.0_f16; }
impl ConstOne for f32  { const ONE: Self = 1.0_f32; }
impl ConstOne for f64  { const ONE: Self = 1.0_f64; }
impl ConstOne for f128 { const ONE: Self = 1.0_f128; }

impl ConstOne for i8   { const ONE: Self = 1_i8; }
impl ConstOne for i16  { const ONE: Self = 1_i16; }
impl ConstOne for i32  { const ONE: Self = 1_i32; }
impl ConstOne for i64  { const ONE: Self = 1_i64; }
impl ConstOne for i128 { const ONE: Self = 1_i128; }

impl ConstOne for u8   { const ONE: Self = 1_u8; }
impl ConstOne for u16  { const ONE: Self = 1_u16; }
impl ConstOne for u32  { const ONE: Self = 1_u32; }
impl ConstOne for u64  { const ONE: Self = 1_u64; }
impl ConstOne for u128 { const ONE: Self = 1_u128; }


pub const trait Zero {
  fn zero() -> Self;
}

const impl Zero for f16  { fn zero() -> Self { 0.0_f16 } }
const impl Zero for f32  { fn zero() -> Self { 0.0_f32 } }
const impl Zero for f64  { fn zero() -> Self { 0.0_f64 } }
const impl Zero for f128 { fn zero() -> Self { 0.0_f128 } }

const impl Zero for i8   { fn zero() -> Self { 0_i8 } }
const impl Zero for i16  { fn zero() -> Self { 0_i16 } }
const impl Zero for i32  { fn zero() -> Self { 0_i32 } }
const impl Zero for i64  { fn zero() -> Self { 0_i64 } }
const impl Zero for i128 { fn zero() -> Self { 0_i128 } }

const impl Zero for u8   { fn zero() -> Self { 0_u8 } }
const impl Zero for u16  { fn zero() -> Self { 0_u16 } }
const impl Zero for u32  { fn zero() -> Self { 0_u32 } }
const impl Zero for u64  { fn zero() -> Self { 0_u64 } }
const impl Zero for u128 { fn zero() -> Self { 0_u128 } }


pub const trait One {
  fn one() -> Self;
}

const impl One for f16  { fn one() -> Self { 1.0_f16 } }
const impl One for f32  { fn one() -> Self { 1.0_f32 } }
const impl One for f64  { fn one() -> Self { 1.0_f64 } }
const impl One for f128 { fn one() -> Self { 1.0_f128 } }

const impl One for i8   { fn one() -> Self { 1_i8 } }
const impl One for i16  { fn one() -> Self { 1_i16 } }
const impl One for i32  { fn one() -> Self { 1_i32 } }
const impl One for i64  { fn one() -> Self { 1_i64 } }
const impl One for i128 { fn one() -> Self { 1_i128 } }

const impl One for u8   { fn one() -> Self { 1_u8 } }
const impl One for u16  { fn one() -> Self { 1_u16 } }
const impl One for u32  { fn one() -> Self { 1_u32 } }
const impl One for u64  { fn one() -> Self { 1_u64 } }
const impl One for u128 { fn one() -> Self { 1_u128 } }


pub trait Sqrt {
  fn sqrt(self) -> Self;
}

impl Sqrt for f16  { fn sqrt(self) -> Self { self.sqrt() } }
impl Sqrt for f32  { fn sqrt(self) -> Self { self.sqrt() } }
impl Sqrt for f64  { fn sqrt(self) -> Self { self.sqrt() } }
impl Sqrt for f128 { fn sqrt(self) -> Self { self.sqrt() } }

impl Sqrt for i8   { fn sqrt(self) -> Self { self.isqrt() } }
impl Sqrt for i16  { fn sqrt(self) -> Self { self.isqrt() } }
impl Sqrt for i32  { fn sqrt(self) -> Self { self.isqrt() } }
impl Sqrt for i64  { fn sqrt(self) -> Self { self.isqrt() } }
impl Sqrt for i128 { fn sqrt(self) -> Self { self.isqrt() } }

impl Sqrt for u8   { fn sqrt(self) -> Self { self.isqrt() } }
impl Sqrt for u16  { fn sqrt(self) -> Self { self.isqrt() } }
impl Sqrt for u32  { fn sqrt(self) -> Self { self.isqrt() } }
impl Sqrt for u64  { fn sqrt(self) -> Self { self.isqrt() } }
impl Sqrt for u128 { fn sqrt(self) -> Self { self.isqrt() } }


pub const trait Min {
  fn min(self, min: Self) -> Self;
}

const impl Min for f16  { fn min(self, min: Self) -> Self { if self < min {min} else {self} } }
const impl Min for f32  { fn min(self, min: Self) -> Self { if self < min {min} else {self} } }
const impl Min for f64  { fn min(self, min: Self) -> Self { if self < min {min} else {self} } }
const impl Min for f128 { fn min(self, min: Self) -> Self { if self < min {min} else {self} } }

const impl Min for i8   { fn min(self, min: Self) -> Self { if self < min {min} else {self} } }
const impl Min for i16  { fn min(self, min: Self) -> Self { if self < min {min} else {self} } }
const impl Min for i32  { fn min(self, min: Self) -> Self { if self < min {min} else {self} } }
const impl Min for i64  { fn min(self, min: Self) -> Self { if self < min {min} else {self} } }
const impl Min for i128 { fn min(self, min: Self) -> Self { if self < min {min} else {self} } }

const impl Min for u8   { fn min(self, min: Self) -> Self { if self < min {min} else {self} } }
const impl Min for u16  { fn min(self, min: Self) -> Self { if self < min {min} else {self} } }
const impl Min for u32  { fn min(self, min: Self) -> Self { if self < min {min} else {self} } }
const impl Min for u64  { fn min(self, min: Self) -> Self { if self < min {min} else {self} } }
const impl Min for u128 { fn min(self, min: Self) -> Self { if self < min {min} else {self} } }


pub const trait Max {
  fn max(self, max: Self) -> Self;
}

const impl Max for f16  { fn max(self, max: Self) -> Self { if self > max {max} else {self} } }
const impl Max for f32  { fn max(self, max: Self) -> Self { if self > max {max} else {self} } }
const impl Max for f64  { fn max(self, max: Self) -> Self { if self > max {max} else {self} } }
const impl Max for f128 { fn max(self, max: Self) -> Self { if self > max {max} else {self} } }

const impl Max for i8   { fn max(self, max: Self) -> Self { if self > max {max} else {self} } }
const impl Max for i16  { fn max(self, max: Self) -> Self { if self > max {max} else {self} } }
const impl Max for i32  { fn max(self, max: Self) -> Self { if self > max {max} else {self} } }
const impl Max for i64  { fn max(self, max: Self) -> Self { if self > max {max} else {self} } }
const impl Max for i128 { fn max(self, max: Self) -> Self { if self > max {max} else {self} } }

const impl Max for u8   { fn max(self, max: Self) -> Self { if self > max {max} else {self} } }
const impl Max for u16  { fn max(self, max: Self) -> Self { if self > max {max} else {self} } }
const impl Max for u32  { fn max(self, max: Self) -> Self { if self > max {max} else {self} } }
const impl Max for u64  { fn max(self, max: Self) -> Self { if self > max {max} else {self} } }
const impl Max for u128 { fn max(self, max: Self) -> Self { if self > max {max} else {self} } }


pub const trait Clamp {
  fn clamp(self, min: Self, max: Self) -> Self;
}

const impl Clamp for f16  { fn clamp(self, min: Self, max: Self) -> Self { if self > max {max} else if self < min {min} else {self} } }
const impl Clamp for f32  { fn clamp(self, min: Self, max: Self) -> Self { if self > max {max} else if self < min {min} else {self} } }
const impl Clamp for f64  { fn clamp(self, min: Self, max: Self) -> Self { if self > max {max} else if self < min {min} else {self} } }
const impl Clamp for f128 { fn clamp(self, min: Self, max: Self) -> Self { if self > max {max} else if self < min {min} else {self} } }

const impl Clamp for i8   { fn clamp(self, min: Self, max: Self) -> Self { if self > max {max} else if self < min {min} else {self} } }
const impl Clamp for i16  { fn clamp(self, min: Self, max: Self) -> Self { if self > max {max} else if self < min {min} else {self} } }
const impl Clamp for i32  { fn clamp(self, min: Self, max: Self) -> Self { if self > max {max} else if self < min {min} else {self} } }
const impl Clamp for i64  { fn clamp(self, min: Self, max: Self) -> Self { if self > max {max} else if self < min {min} else {self} } }
const impl Clamp for i128 { fn clamp(self, min: Self, max: Self) -> Self { if self > max {max} else if self < min {min} else {self} } }

const impl Clamp for u8   { fn clamp(self, min: Self, max: Self) -> Self { if self > max {max} else if self < min {min} else {self} } }
const impl Clamp for u16  { fn clamp(self, min: Self, max: Self) -> Self { if self > max {max} else if self < min {min} else {self} } }
const impl Clamp for u32  { fn clamp(self, min: Self, max: Self) -> Self { if self > max {max} else if self < min {min} else {self} } }
const impl Clamp for u64  { fn clamp(self, min: Self, max: Self) -> Self { if self > max {max} else if self < min {min} else {self} } }
const impl Clamp for u128 { fn clamp(self, min: Self, max: Self) -> Self { if self > max {max} else if self < min {min} else {self} } }


pub const trait Abs {
  fn abs(self) -> Self;
}

const impl Abs for f16  { fn abs(self) -> Self { self.abs() } }
const impl Abs for f32  { fn abs(self) -> Self { self.abs() } }
const impl Abs for f64  { fn abs(self) -> Self { self.abs() } }
const impl Abs for f128 { fn abs(self) -> Self { self.abs() } }

const impl Abs for i8   { fn abs(self) -> Self { self.abs() } }
const impl Abs for i16  { fn abs(self) -> Self { self.abs() } }
const impl Abs for i32  { fn abs(self) -> Self { self.abs() } }
const impl Abs for i64  { fn abs(self) -> Self { self.abs() } }
const impl Abs for i128 { fn abs(self) -> Self { self.abs() } }

const impl Abs for u8   { fn abs(self) -> Self { self } }
const impl Abs for u16  { fn abs(self) -> Self { self } }
const impl Abs for u32  { fn abs(self) -> Self { self } }
const impl Abs for u64  { fn abs(self) -> Self { self } }
const impl Abs for u128 { fn abs(self) -> Self { self } }
