// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use crate::chan::{Ch16, Ch32, Ch64, Ch8};
use core::any::Any;

pub trait Sealed: Any {}
impl Sealed for Ch8 {}
impl Sealed for Ch16 {}
impl Sealed for Ch32 {}
impl Sealed for Ch64 {}
