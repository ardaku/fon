// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

//! Surround Sound 5.1 speaker configuration and types.

use crate::{
    chan::{Ch16, Ch24, Ch32},
    Frame,
};

/// Surround Sound 5.1 audio format (Audio [`Frame`](crate::frame::Frame)
/// containing a front left, rear left, center, lfe, rear right, and front right
/// [`Channel`](crate::chan::Channel)).
pub type Surround<Chan> = Frame<Chan, 6>;

/// 5.1 Surround [16-bit PCM](crate::chan::Ch16) format.
pub type Surround16 = Surround<Ch16>;
/// 5.1 Surround [24-bit Floating Point](crate::chan::Ch24) format.
pub type Surround24 = Surround<Ch24>;
/// 5.1 Surround [32-bit Floating Point](crate::chan::Ch32) format.
pub type Surround32 = Surround<Ch32>;
