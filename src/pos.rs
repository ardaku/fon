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
pub enum Position {
    /// All directions
    ///  - Mono
    Mono,

    /// Low frequency effects (unimportant direction)
    ///  - 5.1
    ///  - 6.1
    ///  - 7.1
    LFE,

    /// Front Left (30 degrees left)
    ///  - 3.0
    ///  - 4.0
    ///  - 5.0
    ///  - 5.1
    ///  - 6.1
    ///  - 7.1
    FLeft,

    /// Front Right (30 degrees right)
    ///  - 3.0
    ///  - 4.0
    ///  - 5.0
    ///  - 5.1
    ///  - 6.1
    ///  - 7.1
    FRight,

    /// Front Center (0 degrees left/right)
    ///  - 3.0
    ///  - 5.0
    ///  - 5.1
    ///  - 6.1
    ///  - 7.1
    Front,

    /// Left Surround (110 degrees left)
    ///  - 4.0
    ///  - 5.0
    ///  - 5.1
    SurroundLeft,

    /// Right Surround (110 degrees right)
    ///  - 4.0
    ///  - 5.0
    ///  - 5.1
    SurroundRight,

    /// Back (180 degrees left/right)
    ///  - 6.1
    Back,

    /// Back Left (150 degrees left)
    ///  - 7.1
    BackLeft,

    /// Back Right (150 degrees right)
    ///  - 7.1
    BackRight,
    
    /// Side Left (90 degrees left)
    ///  - Stereo
    ///  - 6.1
    ///  - 7.1
    SideLeft,
    
    /// Side Right (90 degrees right)
    ///  - Stereo
    ///  - 6.1
    ///  - 7.1
    SideRight,
}

impl From<Position> for f32 {
    fn from(pos: Position) -> f32 {
        match pos {
            Position::Mono => f32::NAN,
            Position::LFE => f32::NAN,
            Position::FLeft => -30.0f32 .to_radians(),
            Position::FRight => 30.0f32 .to_radians(),
            Position::Front => 0.0,
            Position::SurroundLeft => -110.0f32 .to_radians(),
            Position::SurroundRight => 110.0f32 .to_radians(),
            Position::Back => 180.0f32 .to_radians(),
            Position::BackLeft => -150.0f32 .to_radians(),
            Position::BackRight => 150.0f32 .to_radians(),
            Position::SideLeft => -90.0f32 .to_radians(),
            Position::SideRight => 90.0f32 .to_radians(),
        }
    }
}
