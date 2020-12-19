// Copyright Jeron Aldaron Lau 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

use crate::chan::{Ch16, Ch32, Ch64, Ch8};
use core::any::Any;

pub trait Sealed: Any {}
impl Sealed for Ch8 {}
impl Sealed for Ch16 {}
impl Sealed for Ch32 {}
impl Sealed for Ch64 {}
