// Copyright © 2020-2021 The Fon Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use core::ops::Rem;

/// Floating point methods currently only availabe on std, that may be
/// implemented with the libm crate as dependency of core in the future.
pub(crate) trait Libm: Rem<Output = Self> + Sized {
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn abs(self) -> Self;
    fn trunc(self) -> Self;
    fn powi(self, n: i32) -> Self;
    fn rem_euclid(self, rhs: Self) -> Self;
    fn fract(self) -> Self;
}

impl Libm for f32 {
    #[inline(always)]
    fn sin(self) -> Self { libm::sinf(self) }

    #[inline(always)]
    fn cos(self) -> Self { libm::cosf(self) }

    #[inline(always)]
    fn floor(self) -> Self { libm::floorf(self) }

    #[inline(always)]
    fn ceil(self) -> Self { libm::ceilf(self) }

    #[inline(always)]
    fn abs(self) -> Self { libm::fabsf(self) }

    #[inline(always)]
    fn trunc(self) -> Self { libm::truncf(self) }

    #[inline(always)]
    fn powi(self, n: i32) -> Self {
        let mut val = 1.0;
        for _ in 0..n {
            val *= self;
        }
        val
    }

    #[inline(always)]
    fn rem_euclid(self, rhs: Self) -> Self {
        let r = self % rhs;
        if r < 0.0 { r + rhs.abs() } else { r }
    }

    #[inline(always)]
    fn fract(self) -> Self {
        self - self.trunc()
    }
}

impl Libm for f64 {
    #[inline(always)]
    fn sin(self) -> Self { libm::sin(self) }

    #[inline(always)]
    fn cos(self) -> Self { libm::cos(self) }

    #[inline(always)]
    fn floor(self) -> Self { libm::floor(self) }

    #[inline(always)]
    fn ceil(self) -> Self { libm::ceil(self) }

    #[inline(always)]
    fn abs(self) -> Self { libm::fabs(self) }

    #[inline(always)]
    fn trunc(self) -> Self { libm::trunc(self) }

    #[inline(always)]
    fn powi(self, n: i32) -> Self {
        let mut val = 1.0;
        for _ in 0..n {
            val *= self;
        }
        val
    }

    #[inline(always)]
    fn rem_euclid(self, rhs: Self) -> Self {
        let r = self % rhs;
        if r < 0.0 { r + rhs.abs() } else { r }
    }

    #[inline(always)]
    fn fract(self) -> Self {
        self - self.trunc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::f32::consts::PI as PI_F32;
    use core::f64::consts::PI as PI_F64;

    const EPS32: f32 = 0.000016;
    const EPS64: f64 = 0.000016;

    #[test]
    fn powi() {
        for x in [0.0, 1.0, 1.5, -0.4, -1000.09301, 564.33333, PI_F64] {
            for i in 0..5 {
                assert_eq!((Libm::powi(x, i) - f64::powi(x, i)).abs().max(EPS64), EPS64);
            }
        }

        for x in [0.0, 1.0, 1.5, -0.4, -1000.09301, 564.33333, PI_F32] {
            for i in 0..5 {
                assert_eq!((Libm::powi(x, i) - f32::powi(x, i)).abs().max(EPS32), EPS32);
            }
        }
    }

    #[test]
    fn rem_euclid() {
        for x in [0.0, 1.0, 1.5, -0.4, -1000.09301, 564.33333, PI_F64] {
            for y in [3.0, 0.0001, 100.0, -5.0, -0.00001, -250.0] {
                assert_eq!(Libm::rem_euclid(x, y), f64::rem_euclid(x, y));
            }
        }
        
        for x in [0.0, 1.0, 1.5, -0.4, -1000.09301, 564.33333, PI_F32] {
            for y in [3.0, 0.0001, 100.0, -5.0, -0.00001, -250.0] {
                assert_eq!(Libm::rem_euclid(x, y), f32::rem_euclid(x, y));
            }
        }
    }

    #[test]
    fn fract() {
        for x in [0.0, 1.0, 1.5, -0.4, -1000.09301, 564.33333, PI_F64] {
            assert_eq!(Libm::fract(x), f64::fract(x));
        }
        for x in [0.0, 1.0, 1.5, -0.4, -1000.09301, 564.33333, PI_F32] {
            assert_eq!(Libm::fract(x), f32::fract(x));
        }
    }
}
