// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

//! Audio channels (left, right, etc. samples that make up each audio
//! [`Frame`](crate::Frame))

use crate::private::Sealed;
use core::fmt::Debug;
use core::ops::{Add, Mul, Neg, Sub};

/// Component of a speaker configuration, such as *front left*, *lfe*, *etc*.
pub trait Channel:
    Copy
    + Clone
    + Debug
    + Default
    + From<f32>
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Neg<Output = Self>
    + From<Ch16>
    + From<Ch24>
    + From<Ch32>
    + From<Ch64>
    + Into<Ch16>
    + Into<Ch24>
    + Into<Ch32>
    + Into<Ch64>
    + Sealed
    + Unpin
    + Sized
    + 'static
{
    /// Minimum value (*negative one*)
    const MIN: Self;

    /// Mid value (*zero/silence*)
    const MID: Self;

    /// Maximum value (*one*)
    const MAX: Self;

    /// Convert to `f32`
    fn to_f32(self) -> f32;

    /// Linear interpolation
    #[inline(always)]
    fn lerp(self, rhs: Self, t: Self) -> Self {
        self + t * (rhs - self)
    }
}

/// 16-bit sample [Channel](Channel).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Ch16(i16);

impl Channel for Ch16 {
    const MIN: Ch16 = Ch16(-32_768);
    const MID: Ch16 = Ch16(0);
    const MAX: Ch16 = Ch16(32_767);

    #[inline(always)]
    fn to_f32(self) -> f32 {
        (f32::from(self.0) + 0.5) * 32_767.5_f32.recip()
    }
}

impl Ch16 {
    /// Create a new 16-bit [`Channel`](Channel) value.
    #[inline(always)]
    pub const fn new(value: i16) -> Self {
        Self(value)
    }
}

impl From<f32> for Ch16 {
    #[inline(always)]
    fn from(value: f32) -> Self {
        Self::new((value.clamp(-1.0, 1.0) * 32_767.5).floor() as i16)
    }
}

impl From<Ch24> for Ch16 {
    #[inline(always)]
    fn from(ch: Ch24) -> Self {
        Self::new(ch.0)
    }
}

impl From<Ch32> for Ch16 {
    #[inline(always)]
    fn from(ch: Ch32) -> Self {
        Self::from(ch.0)
    }
}

impl From<Ch64> for Ch16 {
    #[inline(always)]
    fn from(ch: Ch64) -> Self {
        Self::from(ch.0 as f32)
    }
}

impl From<Ch16> for i16 {
    #[inline(always)]
    fn from(ch: Ch16) -> i16 {
        ch.0
    }
}

impl<R: Into<Self>> Add<R> for Ch16 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: R) -> Self {
        Self::new(i16::from(self).saturating_add(i16::from(rhs.into())))
    }
}

impl<R: Into<Self>> Sub<R> for Ch16 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: R) -> Self {
        Self::new(i16::from(self).saturating_sub(i16::from(rhs.into())))
    }
}

impl<R: Into<Self>> Mul<R> for Ch16 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: R) -> Self {
        let l = i32::from(self.0);
        let r = i32::from(rhs.into().0);
        let v = (l * r) / 32_767;
        Self::new(v.max(-32_768).min(32_767) as i16)
    }
}

impl Neg for Ch16 {
    type Output = Ch16;

    #[inline(always)]
    fn neg(self) -> Self {
        Self::new((u16::MAX - i16::from(self) as u16) as i16)
    }
}

/// 24-bit sample [Channel](Channel).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[repr(C, packed)]
pub struct Ch24(i16, u8);

impl Channel for Ch24 {
    const MIN: Ch24 = Ch24::new(-8_388_608);
    const MID: Ch24 = Ch24::new(0);
    const MAX: Ch24 = Ch24::new(8_388_607);

    #[inline(always)]
    fn to_f32(self) -> f32 {
        (i32::from(self) as f32 + 0.5) * 8_388_607.5_f32.recip()
    }
}

impl Ch24 {
    /// Create a new 24-bit [`Channel`](Channel) value.
    #[inline(always)]
    pub const fn new(value: i32) -> Self {
        let value = if value < -8_388_608 {
            -8_388_608
        } else if value > 8_388_607 {
            8_388_607
        } else {
            value
        };
        Self((value >> 8) as i16, value as u8)
    }
}

impl From<f32> for Ch24 {
    #[inline(always)]
    fn from(value: f32) -> Self {
        Self::new((value.clamp(-1.0, 1.0) * 8_388_607.5).floor() as i32)
    }
}

impl From<Ch16> for Ch24 {
    #[inline(always)]
    fn from(ch: Ch16) -> Self {
        Self(i16::from(ch), (i16::from(ch) >> 8) as u8 ^ 0b1000_0000)
    }
}

impl From<Ch32> for Ch24 {
    #[inline(always)]
    fn from(ch: Ch32) -> Self {
        Self::from(ch.0)
    }
}

impl From<Ch64> for Ch24 {
    #[inline(always)]
    fn from(ch: Ch64) -> Self {
        Self::from(ch.0 as f32)
    }
}

impl From<Ch24> for i32 {
    #[inline(always)]
    fn from(ch: Ch24) -> i32 {
        ((ch.0 as i32) << 8) | ch.1 as i32
    }
}

impl<R: Into<Self>> Add<R> for Ch24 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: R) -> Self {
        Self::new(i32::from(self) + i32::from(rhs.into()))
    }
}

impl<R: Into<Self>> Sub<R> for Ch24 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: R) -> Self {
        Self::new(i32::from(self) - i32::from(rhs.into()))
    }
}

impl<R: Into<Self>> Mul<R> for Ch24 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: R) -> Self {
        let l: i64 = i32::from(self).into();
        let r: i64 = i32::from(rhs.into()).into();
        let v = (l * r) / 8_388_607;
        Self::new(v.max(-8_388_608).min(8_388_607) as i32)
    }
}

impl Neg for Ch24 {
    type Output = Ch24;

    #[inline(always)]
    fn neg(self) -> Self {
        Self::new((u32::MAX - i32::from(self) as u32) as i32)
    }
}

/// 32-bit sample [Channel](Channel).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Ch32(f32);

impl Channel for Ch32 {
    const MIN: Ch32 = Ch32(-1.0);
    const MID: Ch32 = Ch32(0.0);
    const MAX: Ch32 = Ch32(1.0);

    #[inline(always)]
    fn to_f32(self) -> f32 {
        self.0
    }
}

impl Ch32 {
    /// Create a new 32-bit [`Channel`](Channel) value.
    #[inline(always)]
    pub const fn new(value: f32) -> Self {
        Self(value)
    }
}

impl From<f32> for Ch32 {
    #[inline(always)]
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl From<Ch16> for Ch32 {
    #[inline(always)]
    fn from(ch: Ch16) -> Self {
        Self::new(ch.to_f32())
    }
}

impl From<Ch24> for Ch32 {
    #[inline(always)]
    fn from(ch: Ch24) -> Self {
        Self::new(ch.to_f32())
    }
}

impl From<Ch64> for Ch32 {
    #[inline(always)]
    fn from(ch: Ch64) -> Self {
        Self::new(ch.to_f32())
    }
}

impl From<Ch32> for f32 {
    #[inline(always)]
    fn from(ch: Ch32) -> f32 {
        ch.0
    }
}

impl<R: Into<Self>> Add<R> for Ch32 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: R) -> Self {
        Self::new(f32::from(self) + f32::from(rhs.into()))
    }
}

impl<R: Into<Self>> Sub<R> for Ch32 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: R) -> Self {
        Self::new(f32::from(self) - f32::from(rhs.into()))
    }
}

impl<R: Into<Self>> Mul<R> for Ch32 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: R) -> Self {
        Self::new(f32::from(self) * f32::from(rhs.into()))
    }
}

impl Neg for Ch32 {
    type Output = Ch32;

    #[inline(always)]
    fn neg(self) -> Self {
        Self(-f32::from(self))
    }
}

/// 64-bit sample [Channel](Channel).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Ch64(f64);

impl Channel for Ch64 {
    const MIN: Ch64 = Ch64(-1.0);
    const MID: Ch64 = Ch64(0.0);
    const MAX: Ch64 = Ch64(1.0);

    #[inline(always)]
    fn to_f32(self) -> f32 {
        self.0 as f32
    }
}

impl Ch64 {
    /// Create a new 32-bit [`Channel`](Channel) value.
    #[inline(always)]
    pub const fn new(value: f64) -> Self {
        Self(value)
    }
}

impl From<f32> for Ch64 {
    #[inline(always)]
    fn from(value: f32) -> Self {
        Self::new(value as f64)
    }
}

impl From<Ch16> for Ch64 {
    #[inline(always)]
    fn from(ch: Ch16) -> Self {
        Self::new(ch.to_f32() as f64)
    }
}

impl From<Ch24> for Ch64 {
    #[inline(always)]
    fn from(ch: Ch24) -> Self {
        Self::new(ch.to_f32() as f64)
    }
}

impl From<Ch32> for Ch64 {
    #[inline(always)]
    fn from(ch: Ch32) -> Self {
        Self::new(ch.0 as f64)
    }
}

impl From<Ch64> for f32 {
    #[inline(always)]
    fn from(ch: Ch64) -> f32 {
        ch.0 as f32
    }
}

impl<R: Into<Self>> Add<R> for Ch64 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: R) -> Self {
        Self::new(Ch64::from(self).0 + Ch64::from(rhs.into()).0)
    }
}

impl<R: Into<Self>> Sub<R> for Ch64 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: R) -> Self {
        Self::new(Ch64::from(self).0 - Ch64::from(rhs.into()).0)
    }
}

impl<R: Into<Self>> Mul<R> for Ch64 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: R) -> Self {
        Self::new(Ch64::from(self).0 * Ch64::from(rhs.into()).0)
    }
}

impl Neg for Ch64 {
    type Output = Ch64;

    #[inline(always)]
    fn neg(self) -> Self {
        Self(-Ch64::from(self).0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ch16() {
        assert_eq!(-1.0, Ch16::MIN.to_f32());
        assert_eq!(0.000015259022, Ch16::MID.to_f32());
        assert_eq!(1.0, Ch16::MAX.to_f32());

        assert_eq!(Ch16::MIN, Ch16::from(Ch16::MIN.to_f32()));
        assert_eq!(Ch16::MID, Ch16::from(Ch16::MID.to_f32()));
        assert_eq!(Ch16::MAX, Ch16::from(Ch16::MAX.to_f32()));
    }

    #[test]
    fn ch16_roundtrip() {
        assert_eq!(-32768, i16::from(Ch16::MIN));
        assert_eq!(0, i16::from(Ch16::MID));
        assert_eq!(32767, i16::from(Ch16::MAX));

        assert_eq!(Ch16::MIN, Ch16::new(i16::from(Ch16::MIN)));
        assert_eq!(Ch16::MID, Ch16::new(i16::from(Ch16::MID)));
        assert_eq!(Ch16::MAX, Ch16::new(i16::from(Ch16::MAX)));
    }

    #[test]
    fn ch24() {
        assert_eq!(-1.0, Ch24::MIN.to_f32());
        assert_eq!(0.00000005960465, Ch24::MID.to_f32());
        assert_eq!(1.0, Ch24::MAX.to_f32());

        assert_eq!(Ch24::MIN, Ch24::from(Ch24::MIN.to_f32()));
        assert_eq!(Ch24::MID, Ch24::from(Ch24::MID.to_f32()));
        assert_eq!(Ch24::MAX, Ch24::from(Ch24::MAX.to_f32()));
    }

    #[test]
    fn ch24_roundtrip() {
        assert_eq!(-8388608, i32::from(Ch24::MIN));
        assert_eq!(0, i32::from(Ch24::MID));
        assert_eq!(8388607, i32::from(Ch24::MAX));

        assert_eq!(Ch24::MIN, Ch24::new(i32::from(Ch24::MIN)));
        assert_eq!(Ch24::MID, Ch24::new(i32::from(Ch24::MID)));
        assert_eq!(Ch24::MAX, Ch24::new(i32::from(Ch24::MAX)));
    }

    #[test]
    fn ch32() {
        assert_eq!(-1.0, Ch32::MIN.to_f32());
        assert_eq!(0.0, Ch32::MID.to_f32());
        assert_eq!(1.0, Ch32::MAX.to_f32());

        assert_eq!(Ch32::MIN, Ch32::from(Ch32::MIN.to_f32()));
        assert_eq!(Ch32::MID, Ch32::from(Ch32::MID.to_f32()));
        assert_eq!(Ch32::MAX, Ch32::from(Ch32::MAX.to_f32()));
    }

    #[test]
    fn ch64() {
        assert_eq!(-1.0, Ch64::MIN.to_f32());
        assert_eq!(0.0, Ch64::MID.to_f32());
        assert_eq!(1.0, Ch64::MAX.to_f32());

        assert_eq!(Ch64::MIN, Ch64::from(Ch64::MIN.to_f32()));
        assert_eq!(Ch64::MID, Ch64::from(Ch64::MID.to_f32()));
        assert_eq!(Ch64::MAX, Ch64::from(Ch64::MAX.to_f32()));
    }

    #[test]
    fn ch16_to_ch24() {
        assert_eq!(Ch24::MIN, Ch24::from(Ch16::MIN));
        assert_eq!(Ch24::new(128), Ch24::from(Ch16::MID));
        assert_eq!(Ch24::MAX, Ch24::from(Ch16::MAX));
    }

    #[test]
    fn ch24_to_ch16() {
        assert_eq!(Ch16::MIN, Ch16::from(Ch24::MIN));
        assert_eq!(Ch16::MID, Ch16::from(Ch24::MID));
        assert_eq!(Ch16::MAX, Ch16::from(Ch24::MAX));
    }

    #[test]
    fn ch16_arith() {
        // Test addition
        assert_eq!(Ch16::new(-1), Ch16::new(-32768) + Ch16::new(32767));
        assert_eq!(Ch16::new(8192), Ch16::new(-8192) + Ch16::new(16384));
        assert_eq!(Ch16::MAX, Ch16::MID + Ch16::MAX);
        assert_eq!(Ch16::MIN, Ch16::new(-16384) + Ch16::new(-16384));
        // Test subtraction
        assert_eq!(Ch16::new(0), Ch16::new(-32768) - Ch16::new(-32768));
        assert_eq!(Ch16::new(0), Ch16::new(32767) - Ch16::new(32767));
        assert_eq!(Ch16::new(-32767), Ch16::new(0) - Ch16::new(32767));
        // Test multiplication
        assert_eq!(Ch16::new(0), Ch16::new(0) * Ch16::new(32767));
        assert_eq!(Ch16::new(32767), Ch16::new(32767) * Ch16::new(32767));
        assert_eq!(Ch16::new(-32768), Ch16::new(32767) * Ch16::new(-32768));
        assert_eq!(Ch16::new(-32768), Ch16::new(-32768) * Ch16::new(32767));
        assert_eq!(Ch16::new(32767), Ch16::new(-32768) * Ch16::new(-32768));
        assert_eq!(Ch16::new(-16384), Ch16::new(32767) * Ch16::new(-16384));
        // Test negation
        assert_eq!(Ch16::MIN, -Ch16::MAX);
        assert_eq!(Ch16::MAX, -Ch16::MIN);
        assert_eq!(Ch16::new(-1), -Ch16::new(0));
        assert_eq!(Ch16::new(0), -Ch16::new(-1));
    }

    #[test]
    fn ch24_arith() {
        // Test addition
        assert_eq!(Ch24::new(-1), Ch24::new(-8388608) + Ch24::new(8388607));
        assert_eq!(Ch24::new(2097152), Ch24::new(-2097152) + Ch24::new(4194304));
        assert_eq!(Ch24::MAX, Ch24::MID + Ch24::MAX);
        assert_eq!(Ch24::MIN, Ch24::new(-4194304) + Ch24::new(-4194304));
        // Test subtraction
        assert_eq!(Ch24::new(0), Ch24::new(-8388608) - Ch24::new(-8388608));
        assert_eq!(Ch24::new(0), Ch24::new(8388607) - Ch24::new(8388607));
        assert_eq!(Ch24::new(-8388607), Ch24::new(0) - Ch24::new(8388607));
        // Test multiplication
        assert_eq!(Ch24::new(0), Ch24::new(0) * Ch24::new(8388607));
        assert_eq!(Ch24::new(8388607), Ch24::new(8388607) * Ch24::new(8388607));
        assert_eq!(Ch24::new(-8388608), Ch24::new(8388607) * Ch24::new(-8388608));
        assert_eq!(Ch24::new(-8388608), Ch24::new(-8388608) * Ch24::new(8388607));
        assert_eq!(Ch24::new(8388607), Ch24::new(-8388608) * Ch24::new(-8388608));
        assert_eq!(Ch24::new(-4194304), Ch24::new(8388607) * Ch24::new(-4194304));
        // Test negation
        assert_eq!(Ch24::MIN, -Ch24::MAX);
        assert_eq!(Ch24::MAX, -Ch24::MIN);
        assert_eq!(Ch24::new(-1), -Ch24::new(0));
        assert_eq!(Ch24::new(0), -Ch24::new(-1));
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
        // Test negation
        assert_eq!(Ch32::MIN, -Ch32::MAX);
        assert_eq!(Ch32::MAX, -Ch32::MIN);
        assert_eq!(Ch32::new(0.0), -Ch32::new(0.0));
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
        // Test negation
        assert_eq!(Ch64::MIN, -Ch64::MAX);
        assert_eq!(Ch64::MAX, -Ch64::MIN);
        assert_eq!(Ch64::new(0.0), -Ch64::new(0.0));
    }

    #[test]
    fn ch16_saturation() {
        assert_eq!(Ch16::MAX, Ch16::new(24576) + Ch16::new(16384));
        assert_eq!(Ch16::MIN, Ch16::new(-16384) + Ch16::new(-24576));
        assert_eq!(Ch16::MIN, Ch16::new(-16384) - Ch16::new(24576));
    }

    #[test]
    fn ch24_saturation() {
        assert_eq!(Ch24::MAX, Ch24::new(6291456) + Ch24::new(4194304));
        assert_eq!(Ch24::MIN, Ch24::new(-4194304) + Ch24::new(-6291456));
        assert_eq!(Ch24::MIN, Ch24::new(-4194304) - Ch24::new(6291456));
    }

    #[test]
    fn ch32_unsaturation() {
        assert_eq!(Ch32::new(1.25), Ch32::new(0.75) + Ch32::new(0.5));
        assert_eq!(Ch32::new(-1.25), Ch32::new(-0.5) + Ch32::new(-0.75));
        assert_eq!(Ch32::new(-1.25), Ch32::new(-0.5) - Ch32::new(0.75));
    }

    #[test]
    fn ch64_unsaturation() {
        assert_eq!(Ch64::new(1.25), Ch64::new(0.75) + Ch64::new(0.5));
        assert_eq!(Ch64::new(-1.25), Ch64::new(-0.5) + Ch64::new(-0.75));
        assert_eq!(Ch64::new(-1.25), Ch64::new(-0.5) - Ch64::new(0.75));
    }
}
