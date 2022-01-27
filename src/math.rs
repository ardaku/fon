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
    fn sin(self) -> Self {
        libm::sinf(self)
    }

    #[inline(always)]
    fn cos(self) -> Self {
        libm::cosf(self)
    }

    #[inline(always)]
    fn floor(self) -> Self {
        libm::floorf(self)
    }

    #[inline(always)]
    fn ceil(self) -> Self {
        libm::ceilf(self)
    }

    #[inline(always)]
    fn abs(self) -> Self {
        libm::fabsf(self)
    }

    #[inline(always)]
    fn trunc(self) -> Self {
        libm::truncf(self)
    }

    #[inline(always)]
    fn powi(mut self, n: i32) -> Self {
        match n {
            0 => 1.0,
            i32::MIN => self.powi(i32::MAX) * self,
            x if x < 0 => self.recip().powi(n.wrapping_neg()),
            mut exp => {
                while exp & 1 == 0 {
                    self *= self;
                    exp >>= 1;
                }
                if exp == 1 {
                    return self;
                }
                let mut acc = self;
                while exp > 1 {
                    exp >>= 1;
                    self *= self;
                    if exp & 1 == 1 {
                        acc *= self;
                    }
                }
                acc
            }
        }
    }

    #[inline(always)]
    fn rem_euclid(self, rhs: Self) -> Self {
        let r = self % rhs;
        if r < 0.0 {
            r + rhs.abs()
        } else {
            r
        }
    }

    #[inline(always)]
    fn fract(self) -> Self {
        self - self.trunc()
    }
}

impl Libm for f64 {
    #[inline(always)]
    fn sin(self) -> Self {
        libm::sin(self)
    }

    #[inline(always)]
    fn cos(self) -> Self {
        libm::cos(self)
    }

    #[inline(always)]
    fn floor(self) -> Self {
        libm::floor(self)
    }

    #[inline(always)]
    fn ceil(self) -> Self {
        libm::ceil(self)
    }

    #[inline(always)]
    fn abs(self) -> Self {
        libm::fabs(self)
    }

    #[inline(always)]
    fn trunc(self) -> Self {
        libm::trunc(self)
    }

    #[inline(always)]
    fn powi(mut self, n: i32) -> Self {
        match n {
            0 => 1.0,
            i32::MIN => self.powi(i32::MAX) * self,
            x if x < 0 => self.recip().powi(n.wrapping_neg()),
            mut exp => {
                while exp & 1 == 0 {
                    self *= self;
                    exp >>= 1;
                }
                if exp == 1 {
                    return self;
                }
                let mut acc = self;
                while exp > 1 {
                    exp >>= 1;
                    self *= self;
                    if exp & 1 == 1 {
                        acc *= self;
                    }
                }
                acc
            }
        }
    }

    #[inline(always)]
    fn rem_euclid(self, rhs: Self) -> Self {
        let r = self % rhs;
        if r < 0.0 {
            r + rhs.abs()
        } else {
            r
        }
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

    fn assert_approx_eq_f32(a: f32, b: f32) {
        if a != b {
            let c = ((a - b) / a.min(b)).abs();
            if c.is_infinite() {
                if (a - b).abs() >= 0.000000000000005 {
                    panic!("libm powi(x, i) = {} ≠ std powi() = {}", a, b);
                }
            } else if c >= 1.0000005 {
                panic!("libm powi(x, i) = {} ≠ std powi() = {}", a, b);
            }
        }
    }

    fn assert_approx_eq_f64(a: f64, b: f64) {
        if a != b {
            let c = ((a - b) / a.min(b)).abs();
            if c.is_infinite() {
                if (a - b).abs() >= 0.000000000000005 {
                    panic!("libm powi(x, i) = {} ≠ std powi() = {}", a, b);
                }
            } else if c >= 0.000000000000005 {
                panic!("libm powi(x, i) = {} ≠ std powi() = {}", a, b);
            }
        }
    }

    #[test]
    fn powi() {
        for x in [0.0, 1.0, 1.5, -0.4, -1000.09301, 564.33333, PI_F64] {
            //std implementation has slightly different results across platforms
            for i in -16..16 {
                assert_approx_eq_f64(Libm::powi(x, i), f64::powi(x, i));
            }
        }

        for x in [0.0, 1.0, 1.5, -0.4, -1000.09301, 564.33333, PI_F32] {
            //std implementation has slightly different results across platforms
            for i in -16..16 {
                assert_approx_eq_f32(Libm::powi(x, i), f32::powi(x, i));
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
