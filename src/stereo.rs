// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

//! Stereo speaker configuration and types.

use crate::{
    chan::{Ch16, Ch24, Ch32},
    Frame,
};

/// Stereo audio format (Audio [`Frame`](crate::frame::Frame) containing a left
/// and right [`Channel`](crate::chan::Channel)).
pub type Stereo<Chan> = Frame<Chan, 2>;

/// Stereo [16-bit PCM](crate::chan::Ch16) format.
pub type Stereo16 = Stereo<Ch16>;
/// Stereo [24-bit PCM](crate::chan::Ch24) format.
pub type Stereo24 = Stereo<Ch24>;
/// Stereo [32-bit Floating Point](crate::chan::Ch32) format.
pub type Stereo32 = Stereo<Ch32>;
