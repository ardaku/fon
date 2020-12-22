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
    #[inline(always)]
    fn lerp(self, rhs: Self, t: Self) -> Self {
        self + t * (rhs - self)
    }
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
    #[inline(always)]
    pub fn new(value: i8) -> Self {
        Ch8(value)
    }
}

impl Ch16 {
    /// Create a new 16-bit `Channel` value.
    #[inline(always)]
    pub fn new(value: i16) -> Self {
        Ch16(value)
    }
}

impl Ch32 {
    /// Create a new 32-bit `Channel` value.
    #[inline(always)]
    pub fn new(value: f32) -> Self {
        Ch32(value.min(1.0).max(-1.0))
    }
}

impl Ch64 {
    /// Create a new 64-bit `Channel` value.
    #[inline(always)]
    pub fn new(value: f64) -> Self {
        Ch64(value.min(1.0).max(-1.0))
    }
}

impl From<i8> for Ch8 {
    #[inline(always)]
    fn from(value: i8) -> Self {
        Ch8(value)
    }
}

impl From<Ch8> for i8 {
    #[inline(always)]
    fn from(c: Ch8) -> i8 {
        c.0
    }
}

impl From<i16> for Ch16 {
    #[inline(always)]
    fn from(value: i16) -> Self {
        Ch16(value)
    }
}

impl From<Ch16> for i16 {
    #[inline(always)]
    fn from(c: Ch16) -> i16 {
        c.0
    }
}

impl From<f32> for Ch32 {
    #[inline(always)]
    fn from(value: f32) -> Self {
        Self(value.min(1.0).max(-1.0))
    }
}

impl From<Ch32> for f32 {
    #[inline(always)]
    fn from(c: Ch32) -> f32 {
        c.0
    }
}

impl From<Ch64> for f64 {
    #[inline(always)]
    fn from(c: Ch64) -> f64 {
        c.0
    }
}

// test: ch8_arith()
impl<R: Into<Self>> Sub<R> for Ch8 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: R) -> Self {
        Self(self.0.saturating_sub(rhs.into().0))
    }
}

// test: ch16_arith()
impl<R: Into<Self>> Sub<R> for Ch16 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: R) -> Self {
        Self(self.0.saturating_sub(rhs.into().0))
    }
}

// test: ch32_arith()
impl<R: Into<Self>> Sub<R> for Ch32 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: R) -> Self {
        Self((self.0 - rhs.into().0).min(1.0).max(-1.0))
    }
}

// test: ch64_arith()
impl<R: Into<Self>> Sub<R> for Ch64 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: R) -> Self {
        Self((self.0 - rhs.into().0).min(1.0).max(-1.0))
    }
}

// test: ch8_arith()
impl<R: Into<Self>> Add<R> for Ch8 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: R) -> Self {
        Self(self.0.saturating_add(rhs.into().0))
    }
}

// test: ch16_arith()
impl<R: Into<Self>> Add<R> for Ch16 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: R) -> Self {
        Self(self.0.saturating_add(rhs.into().0))
    }
}

// test: ch32_arith()
impl<R: Into<Self>> Add<R> for Ch32 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: R) -> Self {
        let value = self.0 + rhs.into().0;
        Self(value.min(1.0).max(-1.0))
    }
}

// test: ch64_arith()
impl<R: Into<Self>> Add<R> for Ch64 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: R) -> Self {
        let value = self.0 + rhs.into().0;
        Self(value.min(1.0).max(-1.0))
    }
}

// test: ch8_arith()
impl<R: Into<Self>> Div<R> for Ch8 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: R) -> Self {
        let rhs = rhs.into().0;
        if rhs != 0 {
            let ss = i32::from(self.0) << 8;
            let rr = i32::from(rhs);
            let value = (ss / rr).min(127).max(-128) as i8;
            Self(value)
        } else {
            Self::MAX
        }
    }
}

// test: ch16_arith()
impl<R: Into<Self>> Div<R> for Ch16 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: R) -> Self {
        let rhs = rhs.into().0;
        if rhs != 0 {
            let ss = i32::from(self.0) << 16;
            let rr = i32::from(rhs);
            let value = (ss / rr).min(i16::MAX.into()).max(i16::MIN.into());
            Self(value as i16)
        } else {
            Self::MAX
        }
    }
}

// test: ch32_arith()
impl<R: Into<Self>> Div<R> for Ch32 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: R) -> Self {
        Self((self.0 / rhs.into().0).min(1.0).max(-1.0))
    }
}

// test: ch64_arith()
impl<R: Into<Self>> Div<R> for Ch64 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: R) -> Self {
        Self((self.0 / rhs.into().0).min(1.0).max(-1.0))
    }
}

// test: ch8_arith()
impl<R: Into<Self>> Mul<R> for Ch8 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: R) -> Self {
        let l = i16::from(self.0) + 1;
        let r = i16::from(rhs.into().0);
        let v = (l * r) >> 7;
        Self(v as i8)
    }
}

// test: ch16_arith()
impl<R: Into<Self>> Mul<R> for Ch16 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: R) -> Self {
        let l = i32::from(self.0) + 1;
        let r = i32::from(rhs.into().0);
        let v = (l * r) >> 15;
        Self(v as i16)
    }
}

// test: ch32_arith()
impl<R: Into<Self>> Mul<R> for Ch32 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: R) -> Self {
        Self(self.0 * rhs.into().0)
    }
}

// test: ch64_arith()
impl<R: Into<Self>> Mul<R> for Ch64 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: R) -> Self {
        Self(self.0 * rhs.into().0)
    }
}

// test: See Add
impl<R: Into<Self>> AddAssign<R> for Ch8 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: R) {
        *self = *self + rhs.into();
    }
}

// test: See Add
impl<R: Into<Self>> AddAssign<R> for Ch16 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: R) {
        *self = *self + rhs.into();
    }
}

// test: See Add
impl<R: Into<Self>> AddAssign<R> for Ch32 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: R) {
        *self = *self + rhs.into();
    }
}

// test: See Add
impl<R: Into<Self>> AddAssign<R> for Ch64 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: R) {
        *self = *self + rhs.into();
    }
}

// test: See Sub
impl<R: Into<Self>> SubAssign<R> for Ch8 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: R) {
        *self = *self - rhs.into();
    }
}

// test: See Sub
impl<R: Into<Self>> SubAssign<R> for Ch16 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: R) {
        *self = *self - rhs.into();
    }
}

// test: See Sub
impl<R: Into<Self>> SubAssign<R> for Ch32 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: R) {
        *self = *self - rhs.into();
    }
}

// test: See Sub
impl<R: Into<Self>> SubAssign<R> for Ch64 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: R) {
        *self = *self - rhs.into();
    }
}

// test: See Mul
impl<R: Into<Self>> MulAssign<R> for Ch8 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: R) {
        *self = *self * rhs.into();
    }
}

// test: See Mul
impl<R: Into<Self>> MulAssign<R> for Ch16 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: R) {
        *self = *self * rhs.into();
    }
}

// test: See Mul
impl<R: Into<Self>> MulAssign<R> for Ch32 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: R) {
        *self = *self * rhs.into();
    }
}

// test: See Mul
impl<R: Into<Self>> MulAssign<R> for Ch64 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: R) {
        *self = *self * rhs.into();
    }
}

// test: See Div
impl<R: Into<Self>> DivAssign<R> for Ch8 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: R) {
        *self = *self / rhs.into();
    }
}

// test: See Div
impl<R: Into<Self>> DivAssign<R> for Ch16 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: R) {
        *self = *self / rhs.into();
    }
}

// test: See Div
impl<R: Into<Self>> DivAssign<R> for Ch32 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: R) {
        *self = *self / rhs.into();
    }
}

// test: See Div
impl<R: Into<Self>> DivAssign<R> for Ch64 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: R) {
        *self = *self / rhs.into();
    }
}

// test: all
impl Channel for Ch8 {
    const MIN: Ch8 = Ch8(i8::MIN);
    const MID: Ch8 = Ch8(0);
    const MAX: Ch8 = Ch8(i8::MAX);

    #[inline(always)]
    fn to_f64(self) -> f64 {
        Ch64::from(self).0
    }
}

// test: all
impl Channel for Ch16 {
    const MIN: Ch16 = Ch16(i16::MIN);
    const MID: Ch16 = Ch16(0);
    const MAX: Ch16 = Ch16(i16::MAX);

    #[inline(always)]
    fn to_f64(self) -> f64 {
        Ch64::from(self).0
    }
}

// test: all
impl Channel for Ch32 {
    const MIN: Ch32 = Ch32(-1.0);
    const MID: Ch32 = Ch32(0.0);
    const MAX: Ch32 = Ch32(1.0);

    #[inline(always)]
    fn to_f64(self) -> f64 {
        self.0 as f64
    }
}

// test: all
impl Channel for Ch64 {
    const MIN: Ch64 = Ch64(-1.0);
    const MID: Ch64 = Ch64(0.0);
    const MAX: Ch64 = Ch64(1.0);

    #[inline(always)]
    fn to_f64(self) -> f64 {
        self.0
    }
}

// test:
impl From<f64> for Ch8 {
    #[inline(always)]
    fn from(value: f64) -> Self {
        Ch64::new(value).into()
    }
}

// test:
impl From<f64> for Ch16 {
    #[inline(always)]
    fn from(value: f64) -> Self {
        Ch64::new(value).into()
    }
}

// test:
impl From<f64> for Ch32 {
    #[inline(always)]
    fn from(value: f64) -> Self {
        Ch64::new(value).into()
    }
}

// test:
impl From<f64> for Ch64 {
    #[inline(always)]
    fn from(value: f64) -> Self {
        Ch64::new(value)
    }
}

// test: ch8_roundtrip()
impl From<Ch64> for Ch8 {
    #[inline(always)]
    fn from(value: Ch64) -> Self {
        Ch8::new((value.0 * 127.5 as f64).floor() as i8)
    }
}

// test: ch16_roundtrip()
impl From<Ch64> for Ch16 {
    #[inline(always)]
    fn from(value: Ch64) -> Self {
        Ch16::new((value.0 * 32767.5 as f64).floor() as i16)
    }
}

// test:
impl From<Ch64> for Ch32 {
    #[inline(always)]
    fn from(value: Ch64) -> Self {
        Ch32::new(value.0 as f32)
    }
}

// test: ch8_roundtrip()
impl From<Ch32> for Ch8 {
    #[inline(always)]
    fn from(value: Ch32) -> Self {
        Ch8::new((value.0 * 127.5 as f32).floor() as i8)
    }
}

// test: ch16_roundtrip()
impl From<Ch32> for Ch16 {
    #[inline(always)]
    fn from(value: Ch32) -> Self {
        Ch16::new((value.0 * 32767.5 as f32).floor() as i16)
    }
}

// test: ch32_roundtrip()
impl From<Ch32> for Ch64 {
    #[inline(always)]
    fn from(value: Ch32) -> Self {
        Ch64::new(value.0.into())
    }
}

// test: ch16_to_ch8()
impl From<Ch16> for Ch8 {
    #[inline(always)]
    fn from(c: Ch16) -> Self {
        Ch8::new((c.0 >> 8) as i8)
    }
}

// test: ch16_roundtrip()
impl From<Ch16> for Ch32 {
    #[inline(always)]
    fn from(c: Ch16) -> Self {
        Self((f32::from(c.0) / 32767.5) + (1.0 / 65535.0))
    }
}

// test: ch16_roundtrip()
impl From<Ch16> for Ch64 {
    #[inline(always)]
    fn from(c: Ch16) -> Self {
        Self((f64::from(c.0) / 32767.5) + (1.0 / 65535.0))
    }
}

// test: ch8_to_ch16()
impl From<Ch8> for Ch16 {
    #[inline(always)]
    fn from(c: Ch8) -> Self {
        let c = c.0.wrapping_sub(-128) as u8;
        let v = u16::from_ne_bytes([c, c]).wrapping_add(32768) as i16;
        Ch16::from(v)
    }
}

// test: ch8_roundtrip()
impl From<Ch8> for Ch32 {
    #[inline(always)]
    fn from(c: Ch8) -> Self {
        Self((f32::from(c.0) / 127.5) + (1.0 / 255.0))
    }
}

// test: ch8_roundtrip()
impl From<Ch8> for Ch64 {
    #[inline(always)]
    fn from(c: Ch8) -> Self {
        Self((f64::from(c.0) / 127.5) + (1.0 / 255.0))
    }
}

// test: channel_neg()
impl Neg for Ch8 {
    type Output = Ch8;

    /// Invert sound wave (-x).
    #[inline(always)]
    fn neg(self) -> Self {
        Ch8((u8::MAX - self.0 as u8) as i8)
    }
}

// test: channel_neg()
impl Neg for Ch16 {
    type Output = Ch16;

    /// Invert sound wave (-x).
    #[inline(always)]
    fn neg(self) -> Self {
        Ch16((u16::MAX - self.0 as u16) as i16)
    }
}

// test: channel_neg()
impl Neg for Ch32 {
    type Output = Ch32;

    /// Invert sound wave (-x).
    #[inline(always)]
    fn neg(self) -> Self {
        Ch32(-self.0)
    }
}

// test: channel_neg()
impl Neg for Ch64 {
    type Output = Ch64;

    /// Invert sound wave (-x).
    #[inline(always)]
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
        assert_eq!(1.0, Ch8::new(127).to_f64());

        assert_eq!(Ch8::new(-128), Ch8::from(Ch8::new(-128).to_f64()));
        assert_eq!(Ch8::new(0), Ch8::from(Ch8::new(0).to_f64()));
        assert_eq!(Ch8::new(127), Ch8::from(Ch8::new(127).to_f64()));
    }

    #[test]
    fn ch16_roundtrip() {
        assert_eq!(-1.0, Ch16::new(-32768).to_f64());
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

    #[test]
    fn ch8_to_ch16() {
        assert_eq!(Ch16::new(-32768), Ch16::from(Ch8::new(-128)));
        assert_eq!(Ch16::new(32767), Ch16::from(Ch8::new(127)));
    }

    #[test]
    fn ch16_to_ch8() {
        assert_eq!(Ch8::new(-128), Ch8::from(Ch16::new(-32768)));
        assert_eq!(Ch8::new(0), Ch8::from(Ch16::new(0)));
        assert_eq!(Ch8::new(127), Ch8::from(Ch16::new(32767)));
    }

    #[test]
    fn ch8_arith() {
        // Test addition
        assert_eq!(Ch8::new(-1), Ch8::new(-128) + Ch8::new(127));
        assert_eq!(Ch8::new(32), Ch8::new(-32) + Ch8::new(64));
        assert_eq!(Ch8::new(127), Ch8::new(0) + Ch8::new(127));
        assert_eq!(Ch8::new(-128), Ch8::new(-64) + Ch8::new(-64));
        // Test subtraction
        assert_eq!(Ch8::new(0), Ch8::new(-128) - Ch8::new(-128));
        assert_eq!(Ch8::new(0), Ch8::new(127) - Ch8::new(127));
        assert_eq!(Ch8::new(-127), Ch8::new(0) - Ch8::new(127));
        // Test multiplication
        assert_eq!(Ch8::new(0), Ch8::new(0) * Ch8::new(127));
        assert_eq!(Ch8::new(127), Ch8::new(127) * Ch8::new(127));
        assert_eq!(Ch8::new(-128), Ch8::new(127) * Ch8::new(-128));
        assert_eq!(Ch8::new(127), Ch8::new(-128) * Ch8::new(-128));
        assert_eq!(Ch8::new(-64), Ch8::new(127) * Ch8::new(-64));
        // Test division
        assert_eq!(Ch8::new(0), Ch8::new(0) / Ch8::new(127));
        assert_eq!(Ch8::new(127), Ch8::new(127) / Ch8::new(127));
        assert_eq!(Ch8::new(-128), Ch8::new(127) / Ch8::new(-128));
        assert_eq!(Ch8::new(-128), Ch8::new(-128) / Ch8::new(127));
        assert_eq!(Ch8::new(127), Ch8::new(-128) / Ch8::new(-128));
        assert_eq!(Ch8::new(-128), Ch8::new(64) / Ch8::new(-64));
    }

    #[test]
    fn ch16_arith() {
        // Test addition
        assert_eq!(Ch16::new(-1), Ch16::new(-32768) + Ch16::new(32767));
        assert_eq!(Ch16::new(8192), Ch16::new(-8192) + Ch16::new(16384));
        assert_eq!(Ch16::new(32767), Ch16::new(0) + Ch16::new(32767));
        assert_eq!(Ch16::new(-32768), Ch16::new(-16384) + Ch16::new(-16384));
        // Test subtraction
        assert_eq!(Ch16::new(0), Ch16::new(-32768) - Ch16::new(-32768));
        assert_eq!(Ch16::new(0), Ch16::new(32767) - Ch16::new(32767));
        assert_eq!(Ch16::new(-32767), Ch16::new(0) - Ch16::new(32767));
        // Test multiplication
        assert_eq!(Ch16::new(0), Ch16::new(0) * Ch16::new(32767));
        assert_eq!(Ch16::new(32767), Ch16::new(32767) * Ch16::new(32767));
        assert_eq!(Ch16::new(-32768), Ch16::new(32767) * Ch16::new(-32768));
        assert_eq!(Ch16::new(32767), Ch16::new(-32768) * Ch16::new(-32768));
        assert_eq!(Ch16::new(-16384), Ch16::new(32767) * Ch16::new(-16384));
        // Test division
        assert_eq!(Ch16::new(0), Ch16::new(0) / Ch16::new(32767));
        assert_eq!(Ch16::new(32767), Ch16::new(32767) / Ch16::new(32767));
        assert_eq!(Ch16::new(-32768), Ch16::new(32767) / Ch16::new(-32768));
        assert_eq!(Ch16::new(-32768), Ch16::new(-32768) / Ch16::new(32767));
        assert_eq!(Ch16::new(32767), Ch16::new(-32768) / Ch16::new(-32768));
        assert_eq!(Ch16::new(-32768), Ch16::new(16384) / Ch16::new(-16384));
    }

    #[test]
    fn ch32_arith() {
        // Test addition
        assert_eq!(Ch32::new(0.0), Ch32::new(-1.0) + Ch32::new(1.0));
        assert_eq!(Ch32::new(0.25), Ch32::new(-0.25) + Ch32::new(0.5));
        assert_eq!(Ch32::new(1.0), Ch32::new(0.0) + Ch32::new(1.0));
        assert_eq!(Ch32::new(-1.0), Ch32::new(-0.5) + Ch32::new(-0.5));
        // Test subtraction
        assert_eq!(Ch32::new(0.0), Ch32::new(-1.0) - Ch32::new(-1.0));
        assert_eq!(Ch32::new(0.0), Ch32::new(1.0) - Ch32::new(1.0));
        assert_eq!(Ch32::new(-1.0), Ch32::new(0.0) - Ch32::new(1.0));
        // Test multiplication
        assert_eq!(Ch32::new(0.0), Ch32::new(0.0) * Ch32::new(1.0));
        assert_eq!(Ch32::new(1.0), Ch32::new(1.0) * Ch32::new(1.0));
        assert_eq!(Ch32::new(-1.0), Ch32::new(1.0) * Ch32::new(-1.0));
        assert_eq!(Ch32::new(1.0), Ch32::new(-1.0) * Ch32::new(-1.0));
        assert_eq!(Ch32::new(-0.5), Ch32::new(1.0) * Ch32::new(-0.5));
        // Test division
        assert_eq!(Ch32::new(0.0), Ch32::new(0.0) / Ch32::new(1.0));
        assert_eq!(Ch32::new(1.0), Ch32::new(1.0) / Ch32::new(1.0));
        assert_eq!(Ch32::new(-1.0), Ch32::new(1.0) / Ch32::new(-1.0));
        assert_eq!(Ch32::new(-1.0), Ch32::new(-1.0) / Ch32::new(1.0));
        assert_eq!(Ch32::new(1.0), Ch32::new(-1.0) / Ch32::new(-1.0));
        assert_eq!(Ch32::new(-1.0), Ch32::new(0.5) / Ch32::new(-0.5));
    }

    #[test]
    fn ch64_arith() {
        // Test addition
        assert_eq!(Ch64::new(0.0), Ch64::new(-1.0) + Ch64::new(1.0));
        assert_eq!(Ch64::new(0.25), Ch64::new(-0.25) + Ch64::new(0.5));
        assert_eq!(Ch64::new(1.0), Ch64::new(0.0) + Ch64::new(1.0));
        assert_eq!(Ch64::new(-1.0), Ch64::new(-0.5) + Ch64::new(-0.5));
        // Test subtraction
        assert_eq!(Ch64::new(0.0), Ch64::new(-1.0) - Ch64::new(-1.0));
        assert_eq!(Ch64::new(0.0), Ch64::new(1.0) - Ch64::new(1.0));
        assert_eq!(Ch64::new(-1.0), Ch64::new(0.0) - Ch64::new(1.0));
        // Test multiplication
        assert_eq!(Ch64::new(0.0), Ch64::new(0.0) * Ch64::new(1.0));
        assert_eq!(Ch64::new(1.0), Ch64::new(1.0) * Ch64::new(1.0));
        assert_eq!(Ch64::new(-1.0), Ch64::new(1.0) * Ch64::new(-1.0));
        assert_eq!(Ch64::new(1.0), Ch64::new(-1.0) * Ch64::new(-1.0));
        assert_eq!(Ch64::new(-0.5), Ch64::new(1.0) * Ch64::new(-0.5));
        // Test division
        assert_eq!(Ch64::new(0.0), Ch64::new(0.0) / Ch64::new(1.0));
        assert_eq!(Ch64::new(1.0), Ch64::new(1.0) / Ch64::new(1.0));
        assert_eq!(Ch64::new(-1.0), Ch64::new(1.0) / Ch64::new(-1.0));
        assert_eq!(Ch64::new(-1.0), Ch64::new(-1.0) / Ch64::new(1.0));
        assert_eq!(Ch64::new(1.0), Ch64::new(-1.0) / Ch64::new(-1.0));
        assert_eq!(Ch64::new(-1.0), Ch64::new(0.5) / Ch64::new(-0.5));
    }

    #[test]
    fn ch8_saturation() {
        assert_eq!(Ch8::new(127), Ch8::new(96) + Ch8::new(64));
        assert_eq!(Ch8::new(-128), Ch8::new(-64) + Ch8::new(-96));
        assert_eq!(Ch8::new(-128), Ch8::new(-64) - Ch8::new(96));
        assert_eq!(Ch8::new(127), Ch8::new(64) / Ch8::new(32));
        assert_eq!(Ch8::new(-128), Ch8::new(64) / Ch8::new(-32));
    }

    #[test]
    fn ch16_saturation() {
        assert_eq!(Ch16::new(32767), Ch16::new(24576) + Ch16::new(16384));
        assert_eq!(Ch16::new(-32768), Ch16::new(-16384) + Ch16::new(-24576));
        assert_eq!(Ch16::new(-32768), Ch16::new(-16384) - Ch16::new(24576));
        assert_eq!(Ch16::new(32767), Ch16::new(16384) / Ch16::new(8192));
        assert_eq!(Ch16::new(-32768), Ch16::new(16384) / Ch16::new(-8192));
    }

    #[test]
    fn ch32_saturation() {
        assert_eq!(Ch32::new(1.0), Ch32::new(0.75) + Ch32::new(0.5));
        assert_eq!(Ch32::new(-1.0), Ch32::new(-0.5) + Ch32::new(-0.75));
        assert_eq!(Ch32::new(-1.0), Ch32::new(-0.5) - Ch32::new(0.75));
        assert_eq!(Ch32::new(1.0), Ch32::new(0.5) / Ch32::new(0.25));
        assert_eq!(Ch32::new(-1.0), Ch32::new(0.5) / Ch32::new(-0.25));
    }

    #[test]
    fn ch64_saturation() {
        assert_eq!(Ch64::new(1.0), Ch64::new(0.75) + Ch64::new(0.5));
        assert_eq!(Ch64::new(-1.0), Ch64::new(-0.5) + Ch64::new(-0.75));
        assert_eq!(Ch64::new(-1.0), Ch64::new(-0.5) - Ch64::new(0.75));
        assert_eq!(Ch64::new(1.0), Ch64::new(0.5) / Ch64::new(0.25));
        assert_eq!(Ch64::new(-1.0), Ch64::new(0.5) / Ch64::new(-0.25));
    }
}
