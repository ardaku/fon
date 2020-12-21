// Copyright Jeron Aldaron Lau 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

//! Component channels

use crate::private::Sealed;
use core::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

/// Component of a speaker configuration, such as *front left*, *lfe*, *etc*.
pub trait Channel:
    Copy
    + Debug
    + Default
    + From<f64>
    + PartialOrd
    + Add<Output = Self>
    + Div<Output = Self>
    + Mul<Output = Self>
    + Sub<Output = Self>
    + AddAssign
    + SubAssign
    + DivAssign
    + MulAssign
    + Sealed
    + Unpin
    + From<Ch8>
    + From<Ch16>
    + From<Ch32>
    + From<Ch64>
    + Into<Ch8>
    + Into<Ch16>
    + Into<Ch32>
    + Into<Ch64>
{
    /// Minimum value (*negative one*)
    const MIN: Self;

    /// Mid value (*zero/silence*)
    const MID: Self;

    /// Maximum value (*one*)
    const MAX: Self;

    /// Convert to `f64`
    fn to_f64(self) -> f64;

    /// Linear interpolation
    fn lerp(self, rhs: Self, t: Self) -> Self;
}

/// 8-bit sample [Channel](trait.Channel.html).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Ord, Eq)]
#[repr(transparent)]
pub struct Ch8(i8);

/// 16-bit sample [Channel](trait.Channel.html).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Ord, Eq)]
#[repr(transparent)]
pub struct Ch16(i16);

/// 32-bit sample [Channel](trait.Channel.html).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Ch32(f32);

/// 64-bit sample [Channel](trait.Channel.html).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Ch64(f64);

impl Eq for Ch32 {}

impl Eq for Ch64 {}

impl Ch8 {
    /// Create a new 8-bit `Channel` value.
    pub const fn new(value: i8) -> Self {
        Ch8(value)
    }
}

impl Ch16 {
    /// Create a new 16-bit `Channel` value.
    pub const fn new(value: i16) -> Self {
        Ch16(value)
    }
}

impl Ch32 {
    /// Create a new 32-bit `Channel` value.
    pub const fn new(value: f32) -> Self {
        Ch32(value)
    }
}

impl Ch64 {
    /// Create a new 64-bit `Channel` value.
    pub const fn new(value: f64) -> Self {
        Ch64(value)
    }
}

impl From<i8> for Ch8 {
    fn from(value: i8) -> Self {
        Ch8(value)
    }
}

impl From<Ch8> for i8 {
    fn from(c: Ch8) -> i8 {
        c.0
    }
}

impl From<i16> for Ch16 {
    fn from(value: i16) -> Self {
        Ch16(value)
    }
}

impl From<Ch16> for i16 {
    fn from(c: Ch16) -> i16 {
        c.0
    }
}

impl From<f32> for Ch32 {
    fn from(value: f32) -> Self {
        Ch32(value.min(1.0).max(0.0))
    }
}

impl From<Ch32> for f32 {
    fn from(c: Ch32) -> f32 {
        c.0
    }
}

impl From<Ch64> for f64 {
    fn from(c: Ch64) -> f64 {
        c.0
    }
}

impl<R> Add<R> for Ch8
where
    Self: From<R>,
{
    type Output = Self;
    fn add(self, rhs: R) -> Self {
        let rhs = Self::from(rhs);
        Ch8(self.0.saturating_add(rhs.0))
    }
}

impl<R> AddAssign<R> for Ch8
where
    Self: From<R>,
{
    fn add_assign(&mut self, rhs: R) {
        let rhs = Self::from(rhs);
        self.0 = self.0.saturating_add(rhs.0);
    }
}

impl<R> Sub<R> for Ch8
where
    Self: From<R>,
{
    type Output = Self;
    fn sub(self, rhs: R) -> Self {
        let rhs = Self::from(rhs);
        Ch8(self.0.saturating_sub(rhs.0))
    }
}

impl<R> SubAssign<R> for Ch8
where
    Self: From<R>,
{
    fn sub_assign(&mut self, rhs: R) {
        let rhs = Self::from(rhs);
        self.0 = self.0.saturating_sub(rhs.0);
    }
}

impl<R> Mul<R> for Ch8
where
    Self: From<R>,
{
    type Output = Self;
    fn mul(self, rhs: R) -> Self {
        let rhs = Self::from(rhs);
        let l = i32::from(self.0);
        let l = (l * 16) + (l / 16);
        let r = i32::from(rhs.0);
        let r = (r * 16) + (r / 16);
        let value = ((l * r) / i16::MAX as i32) as i8;
        Ch8(value)
    }
}

impl<R> MulAssign<R> for Ch8
where
    Self: From<R>,
{
    fn mul_assign(&mut self, rhs: R) {
        let rhs = Self::from(rhs);
        self.0 = self.0 * rhs.0;
    }
}

impl<R> Div<R> for Ch8
where
    Self: From<R>,
{
    type Output = Self;
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: R) -> Self {
        let rhs = Self::from(rhs);
        if rhs.0 > 0 {
            let ss = i32::from(self.0) * 256;
            let rr = i32::from(rhs.0);
            let value = (ss / rr).min(255) as i8;
            Ch8(value)
        } else {
            Ch8(0)
        }
    }
}

impl<R> DivAssign<R> for Ch8
where
    Self: From<R>,
{
    fn div_assign(&mut self, rhs: R) {
        let rhs = Self::from(rhs);
        self.0 = self.0 / rhs.0;
    }
}

impl<R> Add<R> for Ch16
where
    Self: From<R>,
{
    type Output = Self;
    fn add(self, rhs: R) -> Self {
        let rhs = Self::from(rhs);
        Ch16(self.0.saturating_add(rhs.0))
    }
}

impl<R> AddAssign<R> for Ch16
where
    Self: From<R>,
{
    fn add_assign(&mut self, rhs: R) {
        let rhs = Self::from(rhs);
        self.0 = self.0 + rhs.0;
    }
}

impl<R> Sub<R> for Ch16
where
    Self: From<R>,
{
    type Output = Self;
    fn sub(self, rhs: R) -> Self {
        let rhs = Self::from(rhs);
        Ch16(self.0.saturating_sub(rhs.0))
    }
}

impl<R> SubAssign<R> for Ch16
where
    Self: From<R>,
{
    fn sub_assign(&mut self, rhs: R) {
        let rhs = Self::from(rhs);
        self.0 = self.0 - rhs.0;
    }
}

impl<R> Mul<R> for Ch16
where
    Self: From<R>,
{
    type Output = Self;
    fn mul(self, rhs: R) -> Self {
        let rhs = Self::from(rhs);
        let l = i64::from(self.0);
        let l = (l * 256) + (l / 256);
        let r = i64::from(rhs.0);
        let r = (r * 256) + (r / 256);
        let value = ((l * r) / u32::MAX as i64) as i16;
        Ch16(value)
    }
}

impl<R> MulAssign<R> for Ch16
where
    Self: From<R>,
{
    fn mul_assign(&mut self, rhs: R) {
        let rhs = Self::from(rhs);
        self.0 = self.0 * rhs.0;
    }
}

impl<R> Div<R> for Ch16
where
    Self: From<R>,
{
    type Output = Self;
    fn div(self, rhs: R) -> Self {
        #![allow(clippy::single_match, clippy::suspicious_arithmetic_impl)]
        let rhs = Self::from(rhs);
        if rhs.0 > 0 {
            let ss = i64::from(self.0) << 16;
            let rr = i64::from(rhs.0);
            let value = (ss / rr).min(i16::MAX.into()) as i16;
            Ch16(value)
        } else {
            Ch16(0)
        }
    }
}

impl<R> DivAssign<R> for Ch16
where
    Self: From<R>,
{
    fn div_assign(&mut self, rhs: R) {
        let rhs = Self::from(rhs);
        self.0 = self.0 / rhs.0;
    }
}

impl<R> Add<R> for Ch32
where
    Self: From<R>,
{
    type Output = Self;
    fn add(self, rhs: R) -> Self {
        let value = self.0 + Self::from(rhs).0;
        Ch32(value.min(1.0))
    }
}

impl<R> AddAssign<R> for Ch32
where
    Self: From<R>,
{
    fn add_assign(&mut self, rhs: R) {
        let rhs = Self::from(rhs);
        self.0 = self.0 + rhs.0;
    }
}

impl<R> Sub<R> for Ch32
where
    Self: From<R>,
{
    type Output = Self;
    fn sub(self, rhs: R) -> Self {
        let value = self.0 - Self::from(rhs).0;
        Ch32(value.max(0.0))
    }
}

impl<R> SubAssign<R> for Ch32
where
    Self: From<R>,
{
    fn sub_assign(&mut self, rhs: R) {
        let rhs = Self::from(rhs);
        self.0 = self.0 - rhs.0;
    }
}

impl<R> Mul<R> for Ch32
where
    Self: From<R>,
{
    type Output = Self;
    fn mul(self, rhs: R) -> Self {
        Ch32(self.0 * Self::from(rhs).0)
    }
}

impl<R> MulAssign<R> for Ch32
where
    Self: From<R>,
{
    fn mul_assign(&mut self, rhs: R) {
        let rhs = Self::from(rhs);
        self.0 = self.0 * rhs.0;
    }
}

impl<R> Div<R> for Ch32
where
    Self: From<R>,
{
    type Output = Self;
    fn div(self, rhs: R) -> Self {
        let v = Self::from(rhs).0;
        if v > 0.0 {
            Ch32((self.0 / v).min(1.0))
        } else {
            Ch32(0.0)
        }
    }
}

impl<R> DivAssign<R> for Ch32
where
    Self: From<R>,
{
    fn div_assign(&mut self, rhs: R) {
        let rhs = Self::from(rhs);
        self.0 = self.0 / rhs.0;
    }
}

impl<R> Add<R> for Ch64
where
    Self: From<R>,
{
    type Output = Self;
    fn add(self, rhs: R) -> Self {
        let value = self.0 + Self::from(rhs).0;
        Ch64(value.min(1.0))
    }
}

impl<R> AddAssign<R> for Ch64
where
    Self: From<R>,
{
    fn add_assign(&mut self, rhs: R) {
        let rhs = Self::from(rhs);
        self.0 = self.0 + rhs.0;
    }
}

impl<R> Sub<R> for Ch64
where
    Self: From<R>,
{
    type Output = Self;
    fn sub(self, rhs: R) -> Self {
        let value = self.0 - Self::from(rhs).0;
        Ch64(value.max(0.0))
    }
}

impl<R> SubAssign<R> for Ch64
where
    Self: From<R>,
{
    fn sub_assign(&mut self, rhs: R) {
        let rhs = Self::from(rhs);
        self.0 = self.0 - rhs.0;
    }
}

impl<R> Mul<R> for Ch64
where
    Self: From<R>,
{
    type Output = Self;
    fn mul(self, rhs: R) -> Self {
        Ch64(self.0 * Self::from(rhs).0)
    }
}

impl<R> MulAssign<R> for Ch64
where
    Self: From<R>,
{
    fn mul_assign(&mut self, rhs: R) {
        let rhs = Self::from(rhs);
        self.0 = self.0 * rhs.0;
    }
}

impl<R> Div<R> for Ch64
where
    Self: From<R>,
{
    type Output = Self;
    fn div(self, rhs: R) -> Self {
        let v = Self::from(rhs).0;
        if v > 0.0 {
            Ch64((self.0 / v).min(1.0))
        } else {
            Ch64(0.0)
        }
    }
}

impl<R> DivAssign<R> for Ch64
where
    Self: From<R>,
{
    fn div_assign(&mut self, rhs: R) {
        let rhs = Self::from(rhs);
        self.0 = self.0 / rhs.0;
    }
}

impl Channel for Ch8 {
    const MIN: Ch8 = Ch8(i8::MIN);
    const MID: Ch8 = Ch8(0);
    const MAX: Ch8 = Ch8(i8::MAX);

    fn to_f64(self) -> f64 {
        Ch64::from(self).0
    }

    /// Linear interpolation
    #[inline]
    fn lerp(self, rhs: Self, t: Self) -> Self {
        let v0: i32 = i8::from(self).into();
        let v1: i32 = i8::from(rhs).into();
        let r = v0 + scale_i32(i8::from(t), v1 - v0);
        Self::new(r as i8)
    }
}

impl Channel for Ch16 {
    const MIN: Ch16 = Ch16(i16::MIN);
    const MID: Ch16 = Ch16(0);
    const MAX: Ch16 = Ch16(i16::MAX);

    fn to_f64(self) -> f64 {
        Ch64::from(self).0
    }

    /// Linear interpolation
    #[inline]
    fn lerp(self, rhs: Self, t: Self) -> Self {
        let v0: i64 = i16::from(self).into();
        let v1: i64 = i16::from(rhs).into();
        let r = v0 + scale_i64(i16::from(t), v1 - v0);
        Self::new(r as i16)
    }
}

impl Channel for Ch32 {
    const MIN: Ch32 = Ch32(-1.0);
    const MID: Ch32 = Ch32(0.0);
    const MAX: Ch32 = Ch32(1.0);

    fn to_f64(self) -> f64 {
        Ch64::from(self).0
    }

    /// Linear interpolation
    #[inline]
    fn lerp(self, rhs: Self, t: Self) -> Self {
        let v0 = f32::from(self);
        let v1 = f32::from(rhs);
        let r = v0 + f32::from(t) * (v1 - v0);
        Self::new(r)
    }
}

impl Channel for Ch64 {
    const MIN: Ch64 = Ch64(-1.0);
    const MID: Ch64 = Ch64(0.0);
    const MAX: Ch64 = Ch64(1.0);

    fn to_f64(self) -> f64 {
        self.0
    }

    /// Linear interpolation
    #[inline]
    fn lerp(self, rhs: Self, t: Self) -> Self {
        let v0 = f64::from(self);
        let v1 = f64::from(rhs);
        let r = v0 + f64::from(t) * (v1 - v0);
        Self::new(r)
    }
}

/// Scale an i32 value by a i8 (for lerp)
#[inline]
fn scale_i32(t: i8, v: i32) -> i32 {
    let c = v * i32::from(t);
    ((c + 1) + (c / 255)) / 255
}

/// Scale an i64 value by a i16 (for lerp)
#[inline]
fn scale_i64(t: i16, v: i64) -> i64 {
    let c = v * i64::from(t);
    ((c + 1) + (c / 65535)) / 65535
}

impl From<f64> for Ch8 {
    fn from(value: f64) -> Self {
        Ch64::new(value).into()
    }
}

impl From<f64> for Ch16 {
    fn from(value: f64) -> Self {
        Ch64::new(value).into()
    }
}

impl From<f64> for Ch32 {
    fn from(value: f64) -> Self {
        Ch64::new(value).into()
    }
}

impl From<f64> for Ch64 {
    fn from(value: f64) -> Self {
        Ch64::new(value)
    }
}

impl From<Ch64> for Ch8 {
    fn from(value: Ch64) -> Self {
        Ch8::new((value.0 * 127.5 as f64).floor() as i8)
    }
}

impl From<Ch64> for Ch16 {
    fn from(value: Ch64) -> Self {
        Ch16::new((value.0 * 32767.5 as f64).floor() as i16)
    }
}

impl From<Ch64> for Ch32 {
    fn from(value: Ch64) -> Self {
        let v: f64 = value.into();
        Ch32::new(v as f32)
    }
}

impl From<Ch32> for Ch8 {
    fn from(value: Ch32) -> Self {
        let value = value.0;
        debug_assert!(value >= -1.0 && value <= 1.0);
        // this cast is not UB since the value is guaranteed
        // to be between -1.0 and 1.0 (see bug #10184)
        Ch8::new((value * i8::MAX as f32).round() as i8)
    }
}

impl From<Ch32> for Ch16 {
    fn from(value: Ch32) -> Self {
        let value = value.0;
        debug_assert!(value >= -1.0 && value <= 1.0);
        // this cast is not UB since the value is guaranteed
        // to be between -1.0 and 1.0 (see bug #10184)
        Ch16::new((value * i16::MAX as f32).round() as i16)
    }
}

impl From<Ch32> for Ch64 {
    fn from(value: Ch32) -> Self {
        let v: f32 = value.into();
        Ch64::new(v.into())
    }
}

impl From<Ch16> for Ch8 {
    fn from(c: Ch16) -> Self {
        Ch8::new((c.0 / 256) as i8)
    }
}

impl From<Ch16> for Ch32 {
    fn from(c: Ch16) -> Self {
        Ch32((f32::from(c.0) / 32767.5).ceil())
    }
}

impl From<Ch16> for Ch64 {
    fn from(c: Ch16) -> Self {
        Ch64((f64::from(c.0) / 32767.5).ceil())
    }
}

impl From<Ch8> for Ch16 {
    fn from(c: Ch8) -> Self {
        let value = i16::from(c.0);
        Ch16::new(value * 256 + value)
    }
}

impl From<Ch8> for Ch32 {
    fn from(c: Ch8) -> Self {
        Ch32((f32::from(c.0) / 127.5).ceil())
    }
}

impl From<Ch8> for Ch64 {
    fn from(c: Ch8) -> Self {
        Ch64((f64::from(c.0) / 127.5).ceil())
    }
}

// test: channel_neg()
impl Neg for Ch8 {
    type Output = Ch8;

    /// Invert sound wave (-x).
    fn neg(self) -> Self {
        Ch8((u8::MAX - self.0 as u8) as i8)
    }
}

// test: channel_neg()
impl Neg for Ch16 {
    type Output = Ch16;

    /// Invert sound wave (-x).
    fn neg(self) -> Self {
        Ch16((u16::MAX - self.0 as u16) as i16)
    }
}

// test: channel_neg()
impl Neg for Ch32 {
    type Output = Ch32;

    /// Invert sound wave (-x).
    fn neg(self) -> Self {
        Ch32(-self.0)
    }
}

// test: channel_neg()
impl Neg for Ch64 {
    type Output = Ch64;

    /// Invert sound wave (-x).
    fn neg(self) -> Self {
        Ch64(-self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_neg() {
        assert_eq!(Ch8::new(-128), -Ch8::new(127));
        assert_eq!(Ch16::new(-32768), -Ch16::new(32767));
        assert_eq!(Ch32::new(-1.0), -Ch32::new(1.0));
        assert_eq!(Ch64::new(-1.0), -Ch64::new(1.0));
        
        assert_eq!(Ch8::new(127), -Ch8::new(-128));
        assert_eq!(Ch16::new(32767), -Ch16::new(-32768));
        assert_eq!(Ch32::new(1.0), -Ch32::new(-1.0));
        assert_eq!(Ch64::new(1.0), -Ch64::new(-1.0));
        
        assert_eq!(Ch8::new(-1), -Ch8::new(0));
        assert_eq!(Ch8::new(0), -Ch8::new(-1));
        assert_eq!(Ch16::new(-1), -Ch16::new(0));
        assert_eq!(Ch16::new(0), -Ch16::new(-1));
        assert_eq!(Ch32::new(0.0), -Ch32::new(0.0));
        assert_eq!(Ch64::new(0.0), -Ch64::new(0.0));
    }

    #[test]
    fn ch8_roundtrip() {
        assert_eq!(-1.0, Ch8::new(-128).to_f64());
        assert_eq!(0.0, Ch8::new(0).to_f64());
        assert_eq!(1.0, Ch8::new(127).to_f64());

        assert_eq!(Ch8::new(-128), Ch8::from(Ch8::new(-128).to_f64()));
        assert_eq!(Ch8::new(0), Ch8::from(Ch8::new(0).to_f64()));
        assert_eq!(Ch8::new(127), Ch8::from(Ch8::new(127).to_f64()));
    }

    #[test]
    fn ch16_roundtrip() {
        assert_eq!(-1.0, Ch16::new(-32768).to_f64());
        assert_eq!(0.0, Ch16::new(0).to_f64());
        assert_eq!(1.0, Ch16::new(32767).to_f64());

        assert_eq!(Ch16::new(-32768), Ch16::from(Ch16::new(-32768).to_f64()));
        assert_eq!(Ch16::new(0), Ch16::from(Ch16::new(0).to_f64()));
        assert_eq!(Ch16::new(32767), Ch16::from(Ch16::new(32767).to_f64()));
    }

    #[test]
    fn ch32_roundtrip() {
        assert_eq!(-1.0, Ch32::new(-1.0).to_f64());
        assert_eq!(0.0, Ch32::new(0.0).to_f64());
        assert_eq!(1.0, Ch32::new(1.0).to_f64());

        assert_eq!(Ch32::new(-1.0), Ch32::from(Ch32::new(-1.0).to_f64()));
        assert_eq!(Ch32::new(0.0), Ch32::from(Ch32::new(0.0).to_f64()));
        assert_eq!(Ch32::new(1.0), Ch32::from(Ch32::new(1.0).to_f64()));
    }
}
