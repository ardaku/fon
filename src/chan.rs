//! Audio samples
//!
//! An audio [`Frame`](crate::frame::Frame) is used to group multiple channels
//! with a positions configuration.

use core::{
    fmt::Debug,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

#[cfg(not(test))]
use crate::math::Libm;
use crate::private::Sealed;

/// Component of a speaker configuration, such as *front left*, *lfe*, *etc*.
pub trait Channel:
    Copy
    + Clone
    + Debug
    + Default
    + From<f32>
    + PartialOrd
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Mul<Output = Self>
    + MulAssign
    + Neg<Output = Self>
    + From<Samp16>
    + From<Samp24>
    + From<Samp32>
    + From<Samp64>
    + Into<Samp16>
    + Into<Samp24>
    + Into<Samp32>
    + Into<Samp64>
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

/// 16-bit audio [`Channel`].
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Samp16(i16);

impl Channel for Samp16 {
    const MAX: Samp16 = Samp16(32_767);
    const MID: Samp16 = Samp16(0);
    const MIN: Samp16 = Samp16(-32_768);

    #[inline(always)]
    fn to_f32(self) -> f32 {
        const MULTIPLIER: f32 = 1.0 / 32_767.5;
        (f32::from(self.0) + 0.5) * MULTIPLIER
    }
}

impl Samp16 {
    /// Create a new 16-bit [`Channel`] value.
    #[inline(always)]
    pub const fn new(value: i16) -> Self {
        Self(value)
    }
}

impl From<f32> for Samp16 {
    #[inline(always)]
    fn from(value: f32) -> Self {
        Self::new((value.clamp(-1.0, 1.0) * 32_767.5).floor() as i16)
    }
}

impl From<Samp24> for Samp16 {
    #[inline(always)]
    fn from(ch: Samp24) -> Self {
        Self::new(ch.0)
    }
}

impl From<Samp32> for Samp16 {
    #[inline(always)]
    fn from(ch: Samp32) -> Self {
        Self::from(ch.0)
    }
}

impl From<Samp64> for Samp16 {
    #[inline(always)]
    fn from(ch: Samp64) -> Self {
        Self::from(ch.0 as f32)
    }
}

impl From<Samp16> for i16 {
    #[inline(always)]
    fn from(ch: Samp16) -> i16 {
        ch.0
    }
}

impl<R: Into<Self>> Add<R> for Samp16 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: R) -> Self {
        Self::new(i16::from(self).saturating_add(i16::from(rhs.into())))
    }
}

impl<R: Into<Self>> Sub<R> for Samp16 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: R) -> Self {
        Self::new(i16::from(self).saturating_sub(i16::from(rhs.into())))
    }
}

impl<R: Into<Self>> Mul<R> for Samp16 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: R) -> Self {
        let l = i32::from(self.0);
        let r = i32::from(rhs.into().0);
        let v = (l * r) / 32_767;
        Self::new(v.clamp(-32_768, 32_767) as i16)
    }
}

impl Neg for Samp16 {
    type Output = Samp16;

    #[inline(always)]
    fn neg(self) -> Self {
        Self::new((u16::MAX - i16::from(self) as u16) as i16)
    }
}

/// 24-bit audio [`Channel`].
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[repr(C, packed)]
pub struct Samp24(i16, u8);

impl Channel for Samp24 {
    const MAX: Samp24 = Samp24::new(8_388_607);
    const MID: Samp24 = Samp24::new(0);
    const MIN: Samp24 = Samp24::new(-8_388_608);

    #[inline(always)]
    fn to_f32(self) -> f32 {
        const MULTIPLIER: f32 = 1.0 / 8_388_607.5;
        (i32::from(self) as f32 + 0.5) * MULTIPLIER
    }
}

impl Samp24 {
    /// Create a new 24-bit [`Channel`] value.
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

impl From<f32> for Samp24 {
    #[inline(always)]
    fn from(value: f32) -> Self {
        Self::new((value.clamp(-1.0, 1.0) * 8_388_607.5).floor() as i32)
    }
}

impl From<Samp16> for Samp24 {
    #[inline(always)]
    fn from(ch: Samp16) -> Self {
        Self(i16::from(ch), (i16::from(ch) >> 8) as u8 ^ 0b1000_0000)
    }
}

impl From<Samp32> for Samp24 {
    #[inline(always)]
    fn from(ch: Samp32) -> Self {
        Self::from(ch.0)
    }
}

impl From<Samp64> for Samp24 {
    #[inline(always)]
    fn from(ch: Samp64) -> Self {
        Self::from(ch.0 as f32)
    }
}

impl From<Samp24> for i32 {
    #[inline(always)]
    fn from(ch: Samp24) -> i32 {
        ((ch.0 as i32) << 8) | ch.1 as i32
    }
}

impl<R: Into<Self>> Add<R> for Samp24 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: R) -> Self {
        Self::new(i32::from(self) + i32::from(rhs.into()))
    }
}

impl<R: Into<Self>> Sub<R> for Samp24 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: R) -> Self {
        Self::new(i32::from(self) - i32::from(rhs.into()))
    }
}

impl<R: Into<Self>> Mul<R> for Samp24 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: R) -> Self {
        let l: i64 = i32::from(self).into();
        let r: i64 = i32::from(rhs.into()).into();
        let v = (l * r) / 8_388_607;
        Self::new(v.clamp(-8_388_608, 8_388_607) as i32)
    }
}

impl Neg for Samp24 {
    type Output = Samp24;

    #[inline(always)]
    fn neg(self) -> Self {
        Self::new((u32::MAX - i32::from(self) as u32) as i32)
    }
}

/// 32-bit audio [`Channel`].
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Samp32(f32);

impl Channel for Samp32 {
    const MAX: Samp32 = Samp32(1.0);
    const MID: Samp32 = Samp32(0.0);
    const MIN: Samp32 = Samp32(-1.0);

    #[inline(always)]
    fn to_f32(self) -> f32 {
        self.0
    }
}

impl Samp32 {
    /// Create a new 32-bit [`Channel`] value.
    #[inline(always)]
    pub const fn new(value: f32) -> Self {
        Self(value)
    }
}

impl From<f32> for Samp32 {
    #[inline(always)]
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl From<Samp16> for Samp32 {
    #[inline(always)]
    fn from(ch: Samp16) -> Self {
        Self::new(ch.to_f32())
    }
}

impl From<Samp24> for Samp32 {
    #[inline(always)]
    fn from(ch: Samp24) -> Self {
        Self::new(ch.to_f32())
    }
}

impl From<Samp64> for Samp32 {
    #[inline(always)]
    fn from(ch: Samp64) -> Self {
        Self::new(ch.to_f32())
    }
}

impl From<Samp32> for f32 {
    #[inline(always)]
    fn from(ch: Samp32) -> f32 {
        ch.0
    }
}

impl<R: Into<Self>> Add<R> for Samp32 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: R) -> Self {
        Self::new(f32::from(self) + f32::from(rhs.into()))
    }
}

impl<R: Into<Self>> Sub<R> for Samp32 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: R) -> Self {
        Self::new(f32::from(self) - f32::from(rhs.into()))
    }
}

impl<R: Into<Self>> Mul<R> for Samp32 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: R) -> Self {
        Self::new(f32::from(self) * f32::from(rhs.into()))
    }
}

impl Neg for Samp32 {
    type Output = Samp32;

    #[inline(always)]
    fn neg(self) -> Self {
        Self(-f32::from(self))
    }
}

/// 64-bit audio [`Channel`].
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Samp64(f64);

impl Channel for Samp64 {
    const MAX: Samp64 = Samp64(1.0);
    const MID: Samp64 = Samp64(0.0);
    const MIN: Samp64 = Samp64(-1.0);

    #[inline(always)]
    fn to_f32(self) -> f32 {
        self.0 as f32
    }
}

impl Samp64 {
    /// Create a new 64-bit [`Channel`] value.
    #[inline(always)]
    pub const fn new(value: f64) -> Self {
        Self(value)
    }
}

impl From<f32> for Samp64 {
    #[inline(always)]
    fn from(value: f32) -> Self {
        Self::new(value as f64)
    }
}

impl From<Samp16> for Samp64 {
    #[inline(always)]
    fn from(ch: Samp16) -> Self {
        Self::new(ch.to_f32() as f64)
    }
}

impl From<Samp24> for Samp64 {
    #[inline(always)]
    fn from(ch: Samp24) -> Self {
        Self::new(ch.to_f32() as f64)
    }
}

impl From<Samp32> for Samp64 {
    #[inline(always)]
    fn from(ch: Samp32) -> Self {
        Self::new(ch.0 as f64)
    }
}

impl From<Samp64> for f64 {
    #[inline(always)]
    fn from(ch: Samp64) -> f64 {
        ch.0
    }
}

impl<R: Into<Self>> Add<R> for Samp64 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: R) -> Self {
        Self::new(self.0 + rhs.into().0)
    }
}

impl<R: Into<Self>> Sub<R> for Samp64 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: R) -> Self {
        Self::new(self.0 - rhs.into().0)
    }
}

impl<R: Into<Self>> Mul<R> for Samp64 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: R) -> Self {
        Self::new(self.0 * rhs.into().0)
    }
}

impl Neg for Samp64 {
    type Output = Samp64;

    #[inline(always)]
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl AddAssign for Samp16 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl AddAssign for Samp24 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl AddAssign for Samp32 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl AddAssign for Samp64 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Samp16 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl SubAssign for Samp24 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl SubAssign for Samp32 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl SubAssign for Samp64 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign for Samp16 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl MulAssign for Samp24 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl MulAssign for Samp32 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl MulAssign for Samp64 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ch16() {
        assert_eq!(-1.0, Samp16::MIN.to_f32());
        assert_eq!(0.000015259022, Samp16::MID.to_f32());
        assert_eq!(1.0, Samp16::MAX.to_f32());

        assert_eq!(Samp16::MIN, Samp16::from(Samp16::MIN.to_f32()));
        assert_eq!(Samp16::MID, Samp16::from(Samp16::MID.to_f32()));
        assert_eq!(Samp16::MAX, Samp16::from(Samp16::MAX.to_f32()));
    }

    #[test]
    fn ch16_roundtrip() {
        assert_eq!(-32768, i16::from(Samp16::MIN));
        assert_eq!(0, i16::from(Samp16::MID));
        assert_eq!(32767, i16::from(Samp16::MAX));

        assert_eq!(Samp16::MIN, Samp16::new(i16::from(Samp16::MIN)));
        assert_eq!(Samp16::MID, Samp16::new(i16::from(Samp16::MID)));
        assert_eq!(Samp16::MAX, Samp16::new(i16::from(Samp16::MAX)));
    }

    #[test]
    fn ch24() {
        assert_eq!(-1.0, Samp24::MIN.to_f32());
        assert_eq!(0.00000005960465, Samp24::MID.to_f32());
        assert_eq!(1.0, Samp24::MAX.to_f32());

        assert_eq!(Samp24::MIN, Samp24::from(Samp24::MIN.to_f32()));
        assert_eq!(Samp24::MID, Samp24::from(Samp24::MID.to_f32()));
        assert_eq!(Samp24::MAX, Samp24::from(Samp24::MAX.to_f32()));
    }

    #[test]
    fn ch24_roundtrip() {
        assert_eq!(-8388608, i32::from(Samp24::MIN));
        assert_eq!(0, i32::from(Samp24::MID));
        assert_eq!(8388607, i32::from(Samp24::MAX));

        assert_eq!(Samp24::MIN, Samp24::new(i32::from(Samp24::MIN)));
        assert_eq!(Samp24::MID, Samp24::new(i32::from(Samp24::MID)));
        assert_eq!(Samp24::MAX, Samp24::new(i32::from(Samp24::MAX)));
    }

    #[test]
    fn ch32() {
        assert_eq!(-1.0, Samp32::MIN.to_f32());
        assert_eq!(0.0, Samp32::MID.to_f32());
        assert_eq!(1.0, Samp32::MAX.to_f32());

        assert_eq!(Samp32::MIN, Samp32::from(Samp32::MIN.to_f32()));
        assert_eq!(Samp32::MID, Samp32::from(Samp32::MID.to_f32()));
        assert_eq!(Samp32::MAX, Samp32::from(Samp32::MAX.to_f32()));
    }

    #[test]
    fn ch64() {
        assert_eq!(-1.0, Samp64::MIN.to_f32());
        assert_eq!(0.0, Samp64::MID.to_f32());
        assert_eq!(1.0, Samp64::MAX.to_f32());

        assert_eq!(Samp64::MIN, Samp64::from(Samp64::MIN.to_f32()));
        assert_eq!(Samp64::MID, Samp64::from(Samp64::MID.to_f32()));
        assert_eq!(Samp64::MAX, Samp64::from(Samp64::MAX.to_f32()));
    }

    #[test]
    fn ch16_to_ch24() {
        assert_eq!(Samp24::MIN, Samp24::from(Samp16::MIN));
        assert_eq!(Samp24::new(128), Samp24::from(Samp16::MID));
        assert_eq!(Samp24::MAX, Samp24::from(Samp16::MAX));
    }

    #[test]
    fn ch24_to_ch16() {
        assert_eq!(Samp16::MIN, Samp16::from(Samp24::MIN));
        assert_eq!(Samp16::MID, Samp16::from(Samp24::MID));
        assert_eq!(Samp16::MAX, Samp16::from(Samp24::MAX));
    }

    #[test]
    fn ch16_arith() {
        // Test addition
        assert_eq!(Samp16::new(-1), Samp16::new(-32768) + Samp16::new(32767));
        assert_eq!(Samp16::new(8192), Samp16::new(-8192) + Samp16::new(16384));
        assert_eq!(Samp16::MAX, Samp16::MID + Samp16::MAX);
        assert_eq!(Samp16::MIN, Samp16::new(-16384) + Samp16::new(-16384));
        // Test subtraction
        assert_eq!(Samp16::new(0), Samp16::new(-32768) - Samp16::new(-32768));
        assert_eq!(Samp16::new(0), Samp16::new(32767) - Samp16::new(32767));
        assert_eq!(Samp16::new(-32767), Samp16::new(0) - Samp16::new(32767));
        // Test multiplication
        assert_eq!(Samp16::new(0), Samp16::new(0) * Samp16::new(32767));
        assert_eq!(Samp16::new(32767), Samp16::new(32767) * Samp16::new(32767));
        assert_eq!(
            Samp16::new(-32768),
            Samp16::new(32767) * Samp16::new(-32768)
        );
        assert_eq!(
            Samp16::new(-32768),
            Samp16::new(-32768) * Samp16::new(32767)
        );
        assert_eq!(
            Samp16::new(32767),
            Samp16::new(-32768) * Samp16::new(-32768)
        );
        assert_eq!(
            Samp16::new(-16384),
            Samp16::new(32767) * Samp16::new(-16384)
        );
        // Test negation
        assert_eq!(Samp16::MIN, -Samp16::MAX);
        assert_eq!(Samp16::MAX, -Samp16::MIN);
        assert_eq!(Samp16::new(-1), -Samp16::new(0));
        assert_eq!(Samp16::new(0), -Samp16::new(-1));
    }

    #[test]
    fn ch24_arith() {
        // Test addition
        assert_eq!(
            Samp24::new(-1),
            Samp24::new(-8388608) + Samp24::new(8388607)
        );
        assert_eq!(
            Samp24::new(2097152),
            Samp24::new(-2097152) + Samp24::new(4194304)
        );
        assert_eq!(Samp24::MAX, Samp24::MID + Samp24::MAX);
        assert_eq!(Samp24::MIN, Samp24::new(-4194304) + Samp24::new(-4194304));
        // Test subtraction
        assert_eq!(
            Samp24::new(0),
            Samp24::new(-8388608) - Samp24::new(-8388608)
        );
        assert_eq!(Samp24::new(0), Samp24::new(8388607) - Samp24::new(8388607));
        assert_eq!(
            Samp24::new(-8388607),
            Samp24::new(0) - Samp24::new(8388607)
        );
        // Test multiplication
        assert_eq!(Samp24::new(0), Samp24::new(0) * Samp24::new(8388607));
        assert_eq!(
            Samp24::new(8388607),
            Samp24::new(8388607) * Samp24::new(8388607)
        );
        assert_eq!(
            Samp24::new(-8388608),
            Samp24::new(8388607) * Samp24::new(-8388608)
        );
        assert_eq!(
            Samp24::new(-8388608),
            Samp24::new(-8388608) * Samp24::new(8388607)
        );
        assert_eq!(
            Samp24::new(8388607),
            Samp24::new(-8388608) * Samp24::new(-8388608)
        );
        assert_eq!(
            Samp24::new(-4194304),
            Samp24::new(8388607) * Samp24::new(-4194304)
        );
        // Test negation
        assert_eq!(Samp24::MIN, -Samp24::MAX);
        assert_eq!(Samp24::MAX, -Samp24::MIN);
        assert_eq!(Samp24::new(-1), -Samp24::new(0));
        assert_eq!(Samp24::new(0), -Samp24::new(-1));
    }

    #[test]
    fn ch32_arith() {
        // Test addition
        assert_eq!(Samp32::new(0.0), Samp32::new(-1.0) + Samp32::new(1.0));
        assert_eq!(Samp32::new(0.25), Samp32::new(-0.25) + Samp32::new(0.5));
        assert_eq!(Samp32::new(1.0), Samp32::new(0.0) + Samp32::new(1.0));
        assert_eq!(Samp32::new(-1.0), Samp32::new(-0.5) + Samp32::new(-0.5));
        // Test subtraction
        assert_eq!(Samp32::new(0.0), Samp32::new(-1.0) - Samp32::new(-1.0));
        assert_eq!(Samp32::new(0.0), Samp32::new(1.0) - Samp32::new(1.0));
        assert_eq!(Samp32::new(-1.0), Samp32::new(0.0) - Samp32::new(1.0));
        // Test multiplication
        assert_eq!(Samp32::new(0.0), Samp32::new(0.0) * Samp32::new(1.0));
        assert_eq!(Samp32::new(1.0), Samp32::new(1.0) * Samp32::new(1.0));
        assert_eq!(Samp32::new(-1.0), Samp32::new(1.0) * Samp32::new(-1.0));
        assert_eq!(Samp32::new(1.0), Samp32::new(-1.0) * Samp32::new(-1.0));
        assert_eq!(Samp32::new(-0.5), Samp32::new(1.0) * Samp32::new(-0.5));
        // Test negation
        assert_eq!(Samp32::MIN, -Samp32::MAX);
        assert_eq!(Samp32::MAX, -Samp32::MIN);
        assert_eq!(Samp32::new(0.0), -Samp32::new(0.0));
    }

    #[test]
    fn ch64_arith() {
        // Test addition
        assert_eq!(Samp64::new(0.0), Samp64::new(-1.0) + Samp64::new(1.0));
        assert_eq!(Samp64::new(0.25), Samp64::new(-0.25) + Samp64::new(0.5));
        assert_eq!(Samp64::new(1.0), Samp64::new(0.0) + Samp64::new(1.0));
        assert_eq!(Samp64::new(-1.0), Samp64::new(-0.5) + Samp64::new(-0.5));
        // Test subtraction
        assert_eq!(Samp64::new(0.0), Samp64::new(-1.0) - Samp64::new(-1.0));
        assert_eq!(Samp64::new(0.0), Samp64::new(1.0) - Samp64::new(1.0));
        assert_eq!(Samp64::new(-1.0), Samp64::new(0.0) - Samp64::new(1.0));
        // Test multiplication
        assert_eq!(Samp64::new(0.0), Samp64::new(0.0) * Samp64::new(1.0));
        assert_eq!(Samp64::new(1.0), Samp64::new(1.0) * Samp64::new(1.0));
        assert_eq!(Samp64::new(-1.0), Samp64::new(1.0) * Samp64::new(-1.0));
        assert_eq!(Samp64::new(1.0), Samp64::new(-1.0) * Samp64::new(-1.0));
        assert_eq!(Samp64::new(-0.5), Samp64::new(1.0) * Samp64::new(-0.5));
        // Test negation
        assert_eq!(Samp64::MIN, -Samp64::MAX);
        assert_eq!(Samp64::MAX, -Samp64::MIN);
        assert_eq!(Samp64::new(0.0), -Samp64::new(0.0));
    }

    #[test]
    fn ch16_saturation() {
        assert_eq!(Samp16::MAX, Samp16::new(24576) + Samp16::new(16384));
        assert_eq!(Samp16::MIN, Samp16::new(-16384) + Samp16::new(-24576));
        assert_eq!(Samp16::MIN, Samp16::new(-16384) - Samp16::new(24576));
    }

    #[test]
    fn ch24_saturation() {
        assert_eq!(Samp24::MAX, Samp24::new(6291456) + Samp24::new(4194304));
        assert_eq!(Samp24::MIN, Samp24::new(-4194304) + Samp24::new(-6291456));
        assert_eq!(Samp24::MIN, Samp24::new(-4194304) - Samp24::new(6291456));
    }

    #[test]
    fn ch32_unsaturation() {
        assert_eq!(Samp32::new(1.25), Samp32::new(0.75) + Samp32::new(0.5));
        assert_eq!(Samp32::new(-1.25), Samp32::new(-0.5) + Samp32::new(-0.75));
        assert_eq!(Samp32::new(-1.25), Samp32::new(-0.5) - Samp32::new(0.75));
    }

    #[test]
    fn ch64_unsaturation() {
        assert_eq!(Samp64::new(1.25), Samp64::new(0.75) + Samp64::new(0.5));
        assert_eq!(Samp64::new(-1.25), Samp64::new(-0.5) + Samp64::new(-0.75));
        assert_eq!(Samp64::new(-1.25), Samp64::new(-0.5) - Samp64::new(0.75));
    }
}
