// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

//! Mono speaker configuration and types.

use crate::{
    chan::{Ch16, Ch24, Ch32},
    Frame,
};

/// Mono audio format (Audio [`Frame`](crate::frame::Frame) containing one
/// [`Channel`](crate::chan::Channel)).
pub type Mono<Chan> = Frame<Chan, 1>;

/// Mono [16-bit PCM](crate::chan::Ch16) format.
pub type Mono16 = Mono<Ch16>;
/// Mono [24-bit Floating Point](crate::chan::Ch24) format.
pub type Mono24 = Mono<Ch24>;
/// Mono [32-bit Floating Point](crate::chan::Ch32) format.
pub type Mono32 = Mono<Ch32>;
