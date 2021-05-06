// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

/// Speaker position.
#[derive(Copy, Clone, Debug)]
pub enum Position {
    /// All directions
    ///  - Mono
    Mono,

    /// Side Left (90 degrees left)
    ///  - Stereo
    ///  - 3.0
    ///  - 6.1
    ///  - 7.1
    Left,

    /// Side Right (90 degrees right)
    ///  - Stereo
    ///  - 3.0
    ///  - 6.1
    ///  - 7.1
    Right,

    /// Center (0/180 degrees left/right)
    ///  - 3.0
    Center,

    /// Front Center (0 degrees left/right)
    ///  - 5.0
    ///  - 5.1
    ///  - 6.1
    ///  - 7.1
    Front,

    /// Front Left (30 degrees left)
    ///  - 3.0
    ///  - 4.0
    ///  - 5.0
    ///  - 5.1
    ///  - 6.1
    ///  - 7.1
    FrontL,

    /// Front Right (30 degrees right)
    ///  - 3.0
    ///  - 4.0
    ///  - 5.0
    ///  - 5.1
    ///  - 6.1
    ///  - 7.1
    FrontR,

    /// Left Surround (110 degrees left)
    ///  - 4.0
    ///  - 5.0
    ///  - 5.1
    SurroundL,

    /// Right Surround (110 degrees right)
    ///  - 4.0
    ///  - 5.0
    ///  - 5.1
    SurroundR,

    /// Low frequency effects (unimportant direction)
    ///  - 5.1
    ///  - 6.1
    ///  - 7.1
    Lfe,

    /// Back (180 degrees left/right)
    ///  - 6.1
    Back,

    /// Back Left (150 degrees left)
    ///  - 7.1
    BackL,

    /// Back Right (150 degrees right)
    ///  - 7.1
    BackR,
}
