// Copyright (c) 2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, or the ZLib
// license <LICENSE-ZLIB or https://www.zlib.net/zlib_license.html> at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Stereo speaker configuration and types.

use crate::{
    chan::{Ch16, Ch32, Ch64, Ch8},
    sample::Sample2,
};

/// Stereo [8-bit PCM](../chan/struct.Ch8.html) format.
pub type Stereo8 = Sample2<Ch8>;
/// Stereo [16-bit PCM](../chan/struct.Ch16.html) format.
pub type Stereo16 = Sample2<Ch16>;
/// Stereo [32-bit Floating Point](../chan/struct.Ch32.html) format.
pub type Stereo32 = Sample2<Ch32>;
/// Stereo [64-bit Floating Point](../chan/struct.Ch64.html) format.
pub type Stereo64 = Sample2<Ch64>;
