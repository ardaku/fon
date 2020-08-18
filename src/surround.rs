// Copyright (c) 2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, or the ZLib
// license <LICENSE-ZLIB or https://www.zlib.net/zlib_license.html> at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Surround speaker configurations and types.
//!
//! # Channel Locations
//!  0. Front Left
//!  1. Front Right
//!  2. Back Left
//!  3. Back Right - Surround 4.0
//!  4. Front Center
//!  5. LFE (low frequency effects) - Surround 5.1
//!  6. Side Left
//!  7. Side Right - Surround 7.1

use crate::{
    chan::{Ch16, Ch32, Ch64, Ch8},
    sample::{Sample6, Sample4, Sample8},
};

/// 4.0 Surround [8-bit PCM](../chan/struct.Ch8.html) format.
pub type Surround4x8 = Sample4<Ch8>;
/// 4.0 Surround [16-bit PCM](../chan/struct.Ch16.html) format.
pub type Surround4x16 = Sample4<Ch16>;
/// 4.0 Surround [32-bit Floating Point](../chan/struct.Ch32.html) format.
pub type Surround4x32 = Sample4<Ch32>;
/// 4.0 Surround [64-bit Floating Point](../chan/struct.Ch64.html) format.
pub type Surround4x64 = Sample4<Ch64>;

/// 5.1 Surround [8-bit PCM](../chan/struct.Ch8.html) format.
pub type Surround5x8 = Sample6<Ch8>;
/// 5.1 Surround [16-bit PCM](../chan/struct.Ch16.html) format.
pub type Surround5x16 = Sample6<Ch16>;
/// 5.1 Surround 32-bit Floating Point](../chan/struct.Ch32.html) format.
pub type Surround5x32 = Sample6<Ch32>;
/// 5.1 Surround [64-bit Floating Point](../chan/struct.Ch64.html) format.
pub type Surround5x64 = Sample6<Ch64>;

/// 7.1 Surround [8-bit PCM](../chan/struct.Ch8.html) format.
pub type Surround7x8 = Sample8<Ch8>;
/// 7.1 Surround [16-bit PCM](../chan/struct.Ch16.html) format.
pub type Surround7x16 = Sample8<Ch16>;
/// 7.1 Surround [32-bit Floating Point](../chan/struct.Ch32.html) format.
pub type Surround7x32 = Sample8<Ch32>;
/// 7.1 Surround [64-bit Floating Point](../chan/struct.Ch64.html) format.
pub type Surround7x64 = Sample8<Ch64>;
