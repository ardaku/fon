// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

//! Rust audio types and conversions.
//!
//! An [audio buffer] can be cheaply converted to and from raw samples (i16, u8,
//! f32, and f64) buffers, enabling interoperability with other crates.
//!
//! Many audio formats are supported:
//!  - Any integer sample rate (32 bits needed to support at least 96_000 Hz)
//!  - Common bit depths (you can use 16-bit to fake 12-/10-/8-bit, as well as
//!    fake unsigned by XOR'ing the top bit)
//!    - [16-bit Signed Integer PCM] (Listening/publishing standard)
//!    - [24-bit Signed Integer PCM] (Older recording/processing standard)
//!    - [32-bit Float PCM] (Newer recording/processing standard)
//!    - [64-bit Float PCM] (Ultra high-quality audio standard)
//!  - Up to 8 channels (following FLAC/SMPTE/ITU-R recommendations):
//!    - 1 Channel: Mono ([Mono])
//!    - 2 Channels: Stereo ([Left], [Right])
//!    - 3 Channels: Surround 3.0 ([Left], [Right], [Center])
//!    - 4 Channels: Surround 4.0 ([FrontL], [FrontR], [SurroundL], [SurroundR])
//!    - 5 Channels: Surround 5.0 ([FrontL], [FrontR], [Front], [SurroundL],
//!      [SurroundR])
//!    - 6 Channels: Surround 5.1 ([FrontL], [FrontR], [Front], [Lfe],
//!      [SurroundL], [SurroundR])
//!    - 7 Channels: Surround 6.1 ([FrontL], [FrontR], [Front], [Lfe], [Back],
//!      [Left], [Right])
//!    - 8 Channels: Surround 7.1 ([FrontL], [FrontR], [Front], [Lfe], [BackL],
//!      [BackR], [Left], [Right])
//!
//! # Getting Started
//! To understand some of the concepts used in this library,
//! [this MDN article] is a good read (although the stuff about compression
//! isn't relevant to this crate's functionality).  This crate uses the MDN
//! definitions for what an audio frame and audio channel are.
//!
//! ## 8-Bit Sawtooth Wave Example
//! ```rust
//! use fon::chan::{Ch16, Ch32};
//! use fon::pos::Mono;
//! use fon::Audio;
//!
//! let mut a = Audio::<Ch32, 1, 44_100>::with_silence(256);
//! let mut counter = 0.0;
//! for f in a.iter_mut() {
//!     f[Mono] = counter.into();
//!     counter += 0.05;
//! }
//!
//! // Convert to stereo 16-Bit 48_000 KHz audio format
//! let mut audio = Audio::<Ch16, 2, 48_000>::with_stream(&a);
//! ```
//!
//! [audio buffer]: crate::Audio
//! [16-bit Signed Integer PCM]: crate::chan::Ch16
//! [24-bit Signed Integer PCM]: crate::chan::Ch24
//! [32-bit Float PCM]: crate::chan::Ch32
//! [64-bit Float PCM]: crate::chan::Ch64
//! [operations]: crate::ops
//! [this MDN article]: https://developer.mozilla.org/en-US/docs/Web/Media/Formats/Audio_concepts
//! [Mono]: crate::pos::Mono
//! [Left]: crate::pos::Left
//! [Right]: crate::pos::Right
//! [Center]: crate::pos::Center
//! [FrontL]: crate::pos::FrontL
//! [FrontR]: crate::pos::FrontR
//! [SurroundL]: crate::pos::SurroundL
//! [SurroundR]: crate::pos::SurroundR
//! [Front]: crate::pos::Front
//! [Lfe]: crate::pos::Lfe
//! [Back]: crate::pos::Back
//! [BackL]: crate::pos::BackL
//! [BackR]: crate::pos::BackR

#![doc(
    html_logo_url = "https://libcala.github.io/logo.svg",
    html_favicon_url = "https://libcala.github.io/icon.svg",
    html_root_url = "https://docs.rs/fon"
)]
// #![deny(unsafe_code)]
#![warn(
    anonymous_parameters,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_qualifications,
    variant_size_differences
)]

extern crate alloc;

mod audio;
mod ops;
mod private;
mod resampler;
mod stream;

pub mod chan;
pub mod frame;
pub mod pos;


pub use audio::Audio;
// pub use streaming::{Resampler, Sink, Stream};

pub use stream::Stream;
pub use resampler::Resampler;
