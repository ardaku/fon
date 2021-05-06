// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use crate::chan::Channel;
use crate::frame::Frame;
use crate::sample::{
    Back, BackL, BackR, Center, Front, FrontL, FrontR, Left, Mono, Right,
    SurroundL, SurroundR,
};
use std::f32::consts::FRAC_PI_2;

/// Trait for mixing a panned channel into a Frame.
pub trait Pan<Chan: Channel> {
    /// Pan the channel.  1.0/0.0 is straight ahead, 0.25 is right, 0.5 is back,
    /// and 0.75 is left.  The algorithm used is "Constant Power Panning".
    fn pan(&mut self, channel: Chan, angle: f32);
}

impl<Chan: Channel> Pan<Chan> for Frame<Chan, 1> {
    #[inline(always)]
    fn pan(&mut self, chan: Chan, _x: f32) {
        self[Mono] = self[Mono] + chan;
    }
}

impl<Chan: Channel> Pan<Chan> for Frame<Chan, 2> {
    #[inline(always)]
    fn pan(&mut self, chan: Chan, x: f32) {
        // Convert to radians, left is now at 0.
        let x = (x + 0.25) * FRAC_PI_2;
        // Pan distance
        self[Left] = self[Left] + chan * x.cos().into();
        self[Right] = self[Right] + chan * x.sin().into();
    }
}

impl<Chan: Channel> Pan<Chan> for Frame<Chan, 3> {
    #[inline(always)]
    fn pan(&mut self, chan: Chan, x: f32) {
        // All nearness distances are 1/4
        match (x.fract() + 1.0).fract() {
            // Center-Right Speakers
            x if x < 0.25 => {
                let x = 4.0 * x * FRAC_PI_2;
                self[Center] = self[Center] + chan * x.cos().into();
                self[Right] = self[Right] + chan * x.sin().into();
            }
            // Right-Center Speakers
            x if x < 0.5 => {
                let x = 4.0 * (x - 0.25) * FRAC_PI_2;
                self[Right] = self[Right] + chan * x.cos().into();
                self[Center] = self[Center] + chan * x.sin().into();
            }
            // Center-Left Speakers
            x if x < 0.75 => {
                let x = 4.0 * (x - 0.50) * FRAC_PI_2;
                self[Center] = self[Center] + chan * x.cos().into();
                self[Left] = self[Left] + chan * x.sin().into();
            }
            // Left-Center Speakers
            x => {
                let x = 4.0 * (x - 0.75) * FRAC_PI_2;
                self[Left] = self[Left] + chan * x.cos().into();
                self[Center] = self[Center] + chan * x.sin().into();
            }
        }
    }
}

impl<Chan: Channel> Pan<Chan> for Frame<Chan, 4> {
    #[inline(always)]
    fn pan(&mut self, chan: Chan, x: f32) {
        // Make 0 be Front Left Speaker
        match (x.fract() + 1.0 + 1.0 / 12.0).fract() {
            // Front Left - Front Right Speakers (60° slice)
            x if x < 60.0 / 360.0 => {
                let x = (360.0 / 60.0) * x * FRAC_PI_2;
                self[FrontL] = self[FrontL] + chan * x.cos().into();
                self[FrontR] = self[FrontR] + chan * x.sin().into();
            }
            // Front Right - Back Right Speakers (80° slice)
            x if x < 140.0 / 360.0 => {
                let x = (360.0 / 80.0) * (x - 60.0 / 360.0) * FRAC_PI_2;
                self[FrontR] = self[FrontR] + chan * x.cos().into();
                self[SurroundR] = self[SurroundR] + chan * x.sin().into();
            }
            // Back Right - Back Left Speakers (140° slice)
            x if x < 280.0 / 360.0 => {
                let x = (360.0 / 140.0) * (x - 140.0 / 360.0) * FRAC_PI_2;
                self[SurroundR] = self[SurroundR] + chan * x.cos().into();
                self[SurroundL] = self[SurroundL] + chan * x.sin().into();
            }
            // Back Left - Front Left Speakers (80° slice)
            x => {
                let x = (360.0 / 80.0) * (x - 280.0 / 360.0) * FRAC_PI_2;
                self[SurroundL] = self[SurroundL] + chan * x.cos().into();
                self[FrontL] = self[FrontL] + chan * x.sin().into();
            }
        }
    }
}

impl<Chan: Channel> Pan<Chan> for Frame<Chan, 5> {
    #[inline(always)]
    fn pan(&mut self, chan: Chan, x: f32) {
        match (x.fract() + 1.0).fract() {
            // Front Center - Front Right Speakers (30° slice)
            x if x < 30.0 / 360.0 => {
                let x = (360.0 / 30.0) * x * FRAC_PI_2;
                self[Front] = self[Front] + chan * x.cos().into();
                self[FrontR] = self[FrontR] + chan * x.sin().into();
            }
            // Front Right - Back Right Speakers (80° slice)
            x if x < 110.0 / 360.0 => {
                let x = (360.0 / 80.0) * (x - 30.0 / 360.0) * FRAC_PI_2;
                self[FrontR] = self[FrontR] + chan * x.cos().into();
                self[SurroundR] = self[SurroundR] + chan * x.sin().into();
            }
            // Back Right - Back Left Speakers (140° slice)
            x if x < 250.0 / 360.0 => {
                let x = (360.0 / 140.0) * (x - 110.0 / 360.0) * FRAC_PI_2;
                self[SurroundR] = self[SurroundR] + chan * x.cos().into();
                self[SurroundL] = self[SurroundL] + chan * x.sin().into();
            }
            // Back Left - Front Left Speakers (80° slice)
            x if x < 330.0 / 360.0 => {
                let x = (360.0 / 80.0) * (x - 250.0 / 360.0) * FRAC_PI_2;
                self[SurroundL] = self[SurroundL] + chan * x.cos().into();
                self[FrontL] = self[FrontL] + chan * x.sin().into();
            }
            // Front Left - Center Speakers (30° slice)
            x => {
                let x = (360.0 / 30.0) * (x - 330.0 / 360.0) * FRAC_PI_2;
                self[FrontL] = self[FrontL] + chan * x.cos().into();
                self[Front] = self[Front] + chan * x.sin().into();
            }
        }
    }
}

impl<Chan: Channel> Pan<Chan> for Frame<Chan, 6> {
    #[inline(always)]
    fn pan(&mut self, chan: Chan, x: f32) {
        match (x.fract() + 1.0).fract() {
            // Front Center - Front Right Speakers (30° slice)
            x if x < 30.0 / 360.0 => {
                let x = (360.0 / 30.0) * x * FRAC_PI_2;
                self[Front] = self[Front] + chan * x.cos().into();
                self[FrontR] = self[FrontR] + chan * x.sin().into();
            }
            // Front Right - Back Right Speakers (80° slice)
            x if x < 110.0 / 360.0 => {
                let x = (360.0 / 80.0) * (x - 30.0 / 360.0) * FRAC_PI_2;
                self[FrontR] = self[FrontR] + chan * x.cos().into();
                self[SurroundR] = self[SurroundR] + chan * x.sin().into();
            }
            // Back Right - Back Left Speakers (140° slice)
            x if x < 250.0 / 360.0 => {
                let x = (360.0 / 140.0) * (x - 110.0 / 360.0) * FRAC_PI_2;
                self[SurroundR] = self[SurroundR] + chan * x.cos().into();
                self[SurroundL] = self[SurroundL] + chan * x.sin().into();
            }
            // Back Left - Front Left Speakers (80° slice)
            x if x < 330.0 / 360.0 => {
                let x = (360.0 / 80.0) * (x - 250.0 / 360.0) * FRAC_PI_2;
                self[SurroundL] = self[SurroundL] + chan * x.cos().into();
                self[FrontL] = self[FrontL] + chan * x.sin().into();
            }
            // Front Left - Center Speakers (30° slice)
            x => {
                let x = (360.0 / 30.0) * (x - 330.0 / 360.0) * FRAC_PI_2;
                self[FrontL] = self[FrontL] + chan * x.cos().into();
                self[Front] = self[Front] + chan * x.sin().into();
            }
        }
    }
}

impl<Chan: Channel> Pan<Chan> for Frame<Chan, 7> {
    #[inline(always)]
    fn pan(&mut self, chan: Chan, x: f32) {
        match (x.fract() + 1.0).fract() {
            // Front Center - Front Right Speakers (30° slice)
            x if x < 30.0 / 360.0 => {
                let x = (360.0 / 30.0) * x * FRAC_PI_2;
                self[Front] = self[Front] + chan * x.cos().into();
                self[FrontR] = self[FrontR] + chan * x.sin().into();
            }
            // Front Right - Side Right Speakers (60° slice)
            x if x < 90.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 30.0 / 360.0) * FRAC_PI_2;
                self[FrontR] = self[FrontR] + chan * x.cos().into();
                self[Right] = self[Right] + chan * x.sin().into();
            }
            // Side Right - Back Speakers (90° slice)
            x if x < 180.0 / 360.0 => {
                let x = (360.0 / 90.0) * (x - 90.0 / 360.0) * FRAC_PI_2;
                self[Right] = self[Right] + chan * x.cos().into();
                self[Back] = self[Back] + chan * x.sin().into();
            }
            // Back - Side Left Speakers (90° slice)
            x if x < 270.0 / 360.0 => {
                let x = (360.0 / 90.0) * (x - 180.0 / 360.0) * FRAC_PI_2;
                self[Back] = self[Back] + chan * x.cos().into();
                self[Left] = self[Left] + chan * x.sin().into();
            }
            // Side Left - Front Left Speakers (60° slice)
            x if x < 330.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 270.0 / 360.0) * FRAC_PI_2;
                self[Left] = self[Left] + chan * x.cos().into();
                self[FrontL] = self[FrontL] + chan * x.sin().into();
            }
            // Front Left - Center Speakers (30° slice)
            x => {
                let x = (360.0 / 30.0) * (x - 330.0 / 360.0) * FRAC_PI_2;
                self[FrontL] = self[FrontL] + chan * x.cos().into();
                self[Front] = self[Front] + chan * x.sin().into();
            }
        }
    }
}

impl<Chan: Channel> Pan<Chan> for Frame<Chan, 8> {
    #[inline(always)]
    fn pan(&mut self, chan: Chan, x: f32) {
        match (x.fract() + 1.0).fract() {
            // Front Center - Front Right Speakers (30° slice)
            x if x < 30.0 / 360.0 => {
                let x = (360.0 / 30.0) * x * FRAC_PI_2;
                self[Front] = self[Front] + chan * x.cos().into();
                self[FrontR] = self[FrontR] + chan * x.sin().into();
            }
            // Front Right - Side Right Speakers (60° slice)
            x if x < 90.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 30.0 / 360.0) * FRAC_PI_2;
                self[FrontR] = self[FrontR] + chan * x.cos().into();
                self[Right] = self[Right] + chan * x.sin().into();
            }
            // Side Right - Back Right Speakers (60° slice)
            x if x < 150.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 90.0 / 360.0) * FRAC_PI_2;
                self[Right] = self[Right] + chan * x.cos().into();
                self[BackR] = self[BackR] + chan * x.sin().into();
            }
            // Back Right - Back Left Speakers (60° slice)
            x if x < 210.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 150.0 / 360.0) * FRAC_PI_2;
                self[BackR] = self[BackR] + chan * x.cos().into();
                self[BackL] = self[BackL] + chan * x.sin().into();
            }
            // Back Left - Side Left Speakers (60° slice)
            x if x < 270.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 210.0 / 360.0) * FRAC_PI_2;
                self[BackL] = self[BackL] + chan * x.cos().into();
                self[Left] = self[Left] + chan * x.sin().into();
            }
            // Side Left - Front Left Speakers (60° slice)
            x if x < 330.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 270.0 / 360.0) * FRAC_PI_2;
                self[Left] = self[Left] + chan * x.cos().into();
                self[FrontL] = self[FrontL] + chan * x.sin().into();
            }
            // Front Left - Center Speakers (30° slice)
            x => {
                let x = (360.0 / 30.0) * (x - 330.0 / 360.0) * FRAC_PI_2;
                self[FrontL] = self[FrontL] + chan * x.cos().into();
                self[Front] = self[Front] + chan * x.sin().into();
            }
        }
    }
}
