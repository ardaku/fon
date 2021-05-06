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
//!    - 1 Channel: [Mono] ([Mono](crate::Position::Mono))
//!    - 2 Channels: [Stereo] ([Left], [Right])
//!    - 3 Channels: [Surround 3.0] ([Left], [Right], [Center])
//!    - 4 Channels: [Surround 4.0] (F.Left, F.Right, B.Left, B.Right)
//!    - 5 Channels: [Surround 5.0] (F.Left, F.Right, F.Center, B.Left, B.Right)
//!    - 6 Channels: [Surround 5.1] (F.Left, F.Right, F.Center, LFE, B.Left,
//!      B.Right)
//!    - 7 Channels: [Surround 6.1] (F.Left, F.Right, F.Center, LFE, B.Center,
//!      S.Left, S.Right)
//!    - 8 Channels: [Surround 7.1] (F.Left, F.Right, F.Center, LFE, B.Left,
//!      B.Right, S.Left, S.Right)
//! 
//! Blending [operations] are supported for all formats.
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
//! use fon::Audio;
//! use fon::frame::Frame;
//! 
//! let mut a = Audio::<Ch32, 1>::with_silence(44_100, 256);
//! let mut counter = 0.0;
//! for s in a.iter_mut() {
//!     s.channels_mut()[0] = Ch32::new(counter);
//!     counter += 0.05;
//! }
//! 
//! // Convert to stereo 16-Bit 48_000 KHz audio format
//! let mut audio = Audio::<Ch16, 2>::with_stream(48_000, &a);
//! ```
//!
//! [audio buffer]: crate::Audio
//! [16-bit Signed Integer PCM]: crate::chan::Ch16
//! [24-bit Signed Integer PCM]: crate::chan::Ch24
//! [32-bit Float PCM]: crate::chan::Ch32
//! [64-bit Float PCM]: crate::chan::Ch64
//! [Mono]: crate::frame::Mono
//! [Stereo]: crate::frame::Stereo
//! [Surround 3.0]: crate::frame::Surround30
//! [Surround 4.0]: crate::frame::Surround40
//! [Surround 5.0]: crate::frame::Surround50
//! [Surround 5.1]: crate::frame::Surround51
//! [Surround 6.1]: crate::frame::Surround61
//! [Surround 7.1]: crate::frame::Surround71
//! [operations]: crate::ops
//! [this MDN article]: https://developer.mozilla.org/en-US/docs/Web/Media/Formats/Audio_concepts
//! [Left]: crate::Position::Left
//! [Right]: crate::Position::Right
//! [Center]: crate::Position::Center

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
mod private;
mod streaming;
mod pos;

// mod resampler;

pub mod chan;
pub mod ops;
pub mod frame;

pub use audio::Audio;
pub use streaming::{Resampler, Sink, Stream};
pub use pos::Position;
