// Copyright (c) 2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, or the ZLib
// license <LICENSE-ZLIB or https://www.zlib.net/zlib_license.html> at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Surround speaker configurations and types.

use crate::{
    chan::{Ch16, Ch32, Ch64, Ch8},
    sample::{Sample6, Sample4, Sample8},
    Config,
};

/// 4 speaker/channel arrangement (4.0 Surround Sound)
///
///  0. Front Left
///  1. Front Right
///  2. Back Left
///  3. Back Right
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Surround4;

impl Config for Surround4 {
    const CHANNEL_COUNT: usize = 4;
}

/// [4.0 Surround](struct.Surround.html) [8-bit PCM](../chan/struct.Ch8.html)
/// format.
pub type Surround4x8 = Sample4<Ch8, Surround4>;
/// [4.0 Surround](struct.Surround.html) [16-bit PCM](../chan/struct.Ch16.html)
/// format.
pub type Surround4x16 = Sample4<Ch16, Surround4>;
/// [4.0 Surround](struct.Surround.html)
/// [32-bit Floating Point](../chan/struct.Ch32.html) format.
pub type Surround4x32 = Sample4<Ch32, Surround4>;
/// [4.0 Surround](struct.Surround.html)
/// [64-bit Floating Point](../chan/struct.Ch64.html) format.
pub type Surround4x64 = Sample4<Ch64, Surround4>;

/// 6 speaker/channel arrangement (ITU 5.1 Surround Sound Standard)
///
///  0. Front Left
///  1. Front Right
///  2. Back Left
///  3. Back Right
///  4. Front Center
///  5. LFE (low frequency effects)
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Surround5;

impl Config for Surround5 {
    const CHANNEL_COUNT: usize = 6;
}

/// [5.1 Surround](struct.Surround.html) [8-bit PCM](../chan/struct.Ch8.html)
/// format.
pub type Surround5x8 = Sample6<Ch8, Surround5>;
/// [5.1 Surround](struct.Surround.html) [16-bit PCM](../chan/struct.Ch16.html)
/// format.
pub type Surround5x16 = Sample6<Ch16, Surround5>;
/// [5.1 Surround](struct.Surround.html)
/// [32-bit Floating Point](../chan/struct.Ch32.html) format.
pub type Surround5x32 = Sample6<Ch32, Surround5>;
/// [5.1 Surround](struct.Surround.html)
/// [64-bit Floating Point](../chan/struct.Ch64.html) format.
pub type Surround5x64 = Sample6<Ch64, Surround5>;

/// 8 speaker/channel arrangement (Blu-ray / Dolby 7.1 Surround Sound Standard)
///
///  0. Front Left
///  1. Front Right
///  2. Back Left
///  3. Back Right
///  4. Front Center
///  5. LFE (low frequency effects)
///  6. Side Left
///  7. Side Right
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Surround7;

impl Config for Surround7 {
    const CHANNEL_COUNT: usize = 8;
}

/// [7.1 Surround](struct.SurroundHD.html) [8-bit PCM](../chan/struct.Ch8.html)
/// format.
pub type Surround7x8 = Sample8<Ch8, Surround7>;
/// [7.1 Surround](struct.SurroundHD.html)
/// [16-bit PCM](../chan/struct.Ch16.html) format.
pub type Surround7x16 = Sample8<Ch16, Surround7>;
/// [7.1 Surround](struct.SurroundHD.html)
/// [32-bit Floating Point](../chan/struct.Ch32.html) format.
pub type Surround7x32 = Sample8<Ch32, Surround7>;
/// [7.1 Surround](struct.SurroundHD.html)
/// [64-bit Floating Point](../chan/struct.Ch64.html) format.
pub type Surround7x64 = Sample8<Ch64, Surround7>;
