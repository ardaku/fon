// Copyright (c) 2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, or the ZLib
// license <LICENSE-ZLIB or https://www.zlib.net/zlib_license.html> at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Rust audio types and conversions.
//!
//! An [audio buffer](struct.Audio.html) can be cheaply converted to and from
//! raw byte buffers, enabling interoperability with other crates.
//!
//! Many audio formats are supported:
//! - Any sample rate
//! - Bit depth: [8]- or [16]-bit integer and [32]- or [64]-bit float
//! - [Mono], [Stereo], [5.1 Surround] and [7.1 Surround]
//!
//! Blending [operations](ops/index.html) are supported for all formats.
//!
//! [8]: chan/struct.Ch8.html
//! [16]: chan/struct.Ch16.html
//! [32]: chan/struct.Ch32.html
//! [64]: chan/struct.Ch64.html
//! [Mono]: mono/struct.Mono.html
//! [Stereo]: stereo/struct.Stereo.html
//! [4.0 Surround]: surround/struct.Surround4.html
//! [5.1 Surround]: surround/struct.Surround5.html
//! [7.1 Surround]: surround/struct.Surround7.html

pub mod chan;
mod private;
mod audio;
mod config;
pub mod mono;
pub mod ops;
pub mod sample;
pub mod stereo;
pub mod surround;

pub use audio::{Audio, Hz};
pub use config::Config;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
