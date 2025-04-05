//! Speaker/channel positions within a speaker configuration.

use core::ops::{Index, IndexMut};

use crate::{chan::Channel, frame::Frame};

/// All directions
///  - Mono
#[derive(Copy, Clone, Debug)]
pub struct Mono;

/// Side Left (90 degrees left)
///  - Stereo
///  - 3.0
///  - 6.1
///  - 7.1
#[derive(Copy, Clone, Debug)]
pub struct Left;

/// Side Right (90 degrees right)
///  - Stereo
///  - 3.0
///  - 6.1
///  - 7.1
#[derive(Copy, Clone, Debug)]
pub struct Right;

/// Center (0/180 degrees left/right)
///  - 3.0
#[derive(Copy, Clone, Debug)]
pub struct Center;

/// Front Center (0 degrees left/right)
///  - 5.0
///  - 5.1
///  - 6.1
///  - 7.1
#[derive(Copy, Clone, Debug)]
pub struct Front;

/// Front Left (30 degrees left)
///  - 3.0
///  - 4.0
///  - 5.0
///  - 5.1
///  - 6.1
///  - 7.1
#[derive(Copy, Clone, Debug)]
pub struct FrontL;

/// Front Right (30 degrees right)
///  - 3.0
///  - 4.0
///  - 5.0
///  - 5.1
///  - 6.1
///  - 7.1
#[derive(Copy, Clone, Debug)]
pub struct FrontR;

/// Left Surround (110 degrees left)
///  - 4.0
///  - 5.0
///  - 5.1
#[derive(Copy, Clone, Debug)]
pub struct SurroundL;

/// Right Surround (110 degrees right)
///  - 4.0
///  - 5.0
///  - 5.1
#[derive(Copy, Clone, Debug)]
pub struct SurroundR;

/// Low frequency effects (unimportant direction)
///  - 5.1
///  - 6.1
///  - 7.1
#[derive(Copy, Clone, Debug)]
pub struct Lfe;

/// Back (180 degrees left/right)
///  - 6.1
#[derive(Copy, Clone, Debug)]
pub struct Back;

/// Back Left (150 degrees left)
///  - 7.1
#[derive(Copy, Clone, Debug)]
pub struct BackL;

/// Back Right (150 degrees right)
///  - 7.1
#[derive(Copy, Clone, Debug)]
pub struct BackR;

////////////////////////////////////////////////////////////

impl<C: Channel> Index<Mono> for Frame<C, 1> {
    type Output = C;

    fn index(&self, _: Mono) -> &Self::Output {
        &self.samples()[0]
    }
}

impl<C: Channel> IndexMut<Mono> for Frame<C, 1> {
    fn index_mut(&mut self, _: Mono) -> &mut Self::Output {
        &mut self.samples_mut()[0]
    }
}

////////////////////////////////////////////////////////////

impl<C: Channel> Index<Left> for Frame<C, 2> {
    type Output = C;

    fn index(&self, _: Left) -> &Self::Output {
        &self.samples()[0]
    }
}

impl<C: Channel> IndexMut<Left> for Frame<C, 2> {
    fn index_mut(&mut self, _: Left) -> &mut Self::Output {
        &mut self.samples_mut()[0]
    }
}

impl<C: Channel> Index<Right> for Frame<C, 2> {
    type Output = C;

    fn index(&self, _: Right) -> &Self::Output {
        &self.samples()[1]
    }
}

impl<C: Channel> IndexMut<Right> for Frame<C, 2> {
    fn index_mut(&mut self, _: Right) -> &mut Self::Output {
        &mut self.samples_mut()[1]
    }
}

////////////////////////////////////////////////////////////

impl<C: Channel> Index<Left> for Frame<C, 3> {
    type Output = C;

    fn index(&self, _: Left) -> &Self::Output {
        &self.samples()[0]
    }
}

impl<C: Channel> IndexMut<Left> for Frame<C, 3> {
    fn index_mut(&mut self, _: Left) -> &mut Self::Output {
        &mut self.samples_mut()[0]
    }
}

impl<C: Channel> Index<Right> for Frame<C, 3> {
    type Output = C;

    fn index(&self, _: Right) -> &Self::Output {
        &self.samples()[1]
    }
}

impl<C: Channel> IndexMut<Right> for Frame<C, 3> {
    fn index_mut(&mut self, _: Right) -> &mut Self::Output {
        &mut self.samples_mut()[1]
    }
}

impl<C: Channel> Index<Center> for Frame<C, 3> {
    type Output = C;

    fn index(&self, _: Center) -> &Self::Output {
        &self.samples()[2]
    }
}

impl<C: Channel> IndexMut<Center> for Frame<C, 3> {
    fn index_mut(&mut self, _: Center) -> &mut Self::Output {
        &mut self.samples_mut()[2]
    }
}

////////////////////////////////////////////////////////////

impl<C: Channel> Index<FrontL> for Frame<C, 4> {
    type Output = C;

    fn index(&self, _: FrontL) -> &Self::Output {
        &self.samples()[0]
    }
}

impl<C: Channel> IndexMut<FrontL> for Frame<C, 4> {
    fn index_mut(&mut self, _: FrontL) -> &mut Self::Output {
        &mut self.samples_mut()[0]
    }
}

impl<C: Channel> Index<FrontR> for Frame<C, 4> {
    type Output = C;

    fn index(&self, _: FrontR) -> &Self::Output {
        &self.samples()[1]
    }
}

impl<C: Channel> IndexMut<FrontR> for Frame<C, 4> {
    fn index_mut(&mut self, _: FrontR) -> &mut Self::Output {
        &mut self.samples_mut()[1]
    }
}

impl<C: Channel> Index<SurroundL> for Frame<C, 4> {
    type Output = C;

    fn index(&self, _: SurroundL) -> &Self::Output {
        &self.samples()[2]
    }
}

impl<C: Channel> IndexMut<SurroundL> for Frame<C, 4> {
    fn index_mut(&mut self, _: SurroundL) -> &mut Self::Output {
        &mut self.samples_mut()[2]
    }
}

impl<C: Channel> Index<SurroundR> for Frame<C, 4> {
    type Output = C;

    fn index(&self, _: SurroundR) -> &Self::Output {
        &self.samples()[3]
    }
}

impl<C: Channel> IndexMut<SurroundR> for Frame<C, 4> {
    fn index_mut(&mut self, _: SurroundR) -> &mut Self::Output {
        &mut self.samples_mut()[3]
    }
}

////////////////////////////////////////////////////////////

impl<C: Channel> Index<FrontL> for Frame<C, 5> {
    type Output = C;

    fn index(&self, _: FrontL) -> &Self::Output {
        &self.samples()[0]
    }
}

impl<C: Channel> IndexMut<FrontL> for Frame<C, 5> {
    fn index_mut(&mut self, _: FrontL) -> &mut Self::Output {
        &mut self.samples_mut()[0]
    }
}

impl<C: Channel> Index<FrontR> for Frame<C, 5> {
    type Output = C;

    fn index(&self, _: FrontR) -> &Self::Output {
        &self.samples()[1]
    }
}

impl<C: Channel> IndexMut<FrontR> for Frame<C, 5> {
    fn index_mut(&mut self, _: FrontR) -> &mut Self::Output {
        &mut self.samples_mut()[1]
    }
}

impl<C: Channel> Index<Front> for Frame<C, 5> {
    type Output = C;

    fn index(&self, _: Front) -> &Self::Output {
        &self.samples()[2]
    }
}

impl<C: Channel> IndexMut<Front> for Frame<C, 5> {
    fn index_mut(&mut self, _: Front) -> &mut Self::Output {
        &mut self.samples_mut()[2]
    }
}

impl<C: Channel> Index<SurroundL> for Frame<C, 5> {
    type Output = C;

    fn index(&self, _: SurroundL) -> &Self::Output {
        &self.samples()[3]
    }
}

impl<C: Channel> IndexMut<SurroundL> for Frame<C, 5> {
    fn index_mut(&mut self, _: SurroundL) -> &mut Self::Output {
        &mut self.samples_mut()[3]
    }
}

impl<C: Channel> Index<SurroundR> for Frame<C, 5> {
    type Output = C;

    fn index(&self, _: SurroundR) -> &Self::Output {
        &self.samples()[4]
    }
}

impl<C: Channel> IndexMut<SurroundR> for Frame<C, 5> {
    fn index_mut(&mut self, _: SurroundR) -> &mut Self::Output {
        &mut self.samples_mut()[4]
    }
}

////////////////////////////////////////////////////////////

impl<C: Channel> Index<FrontL> for Frame<C, 6> {
    type Output = C;

    fn index(&self, _: FrontL) -> &Self::Output {
        &self.samples()[0]
    }
}

impl<C: Channel> IndexMut<FrontL> for Frame<C, 6> {
    fn index_mut(&mut self, _: FrontL) -> &mut Self::Output {
        &mut self.samples_mut()[0]
    }
}

impl<C: Channel> Index<FrontR> for Frame<C, 6> {
    type Output = C;

    fn index(&self, _: FrontR) -> &Self::Output {
        &self.samples()[1]
    }
}

impl<C: Channel> IndexMut<FrontR> for Frame<C, 6> {
    fn index_mut(&mut self, _: FrontR) -> &mut Self::Output {
        &mut self.samples_mut()[1]
    }
}

impl<C: Channel> Index<Front> for Frame<C, 6> {
    type Output = C;

    fn index(&self, _: Front) -> &Self::Output {
        &self.samples()[2]
    }
}

impl<C: Channel> IndexMut<Front> for Frame<C, 6> {
    fn index_mut(&mut self, _: Front) -> &mut Self::Output {
        &mut self.samples_mut()[2]
    }
}

impl<C: Channel> Index<Lfe> for Frame<C, 6> {
    type Output = C;

    fn index(&self, _: Lfe) -> &Self::Output {
        &self.samples()[3]
    }
}

impl<C: Channel> IndexMut<Lfe> for Frame<C, 6> {
    fn index_mut(&mut self, _: Lfe) -> &mut Self::Output {
        &mut self.samples_mut()[3]
    }
}

impl<C: Channel> Index<SurroundL> for Frame<C, 6> {
    type Output = C;

    fn index(&self, _: SurroundL) -> &Self::Output {
        &self.samples()[4]
    }
}

impl<C: Channel> IndexMut<SurroundL> for Frame<C, 6> {
    fn index_mut(&mut self, _: SurroundL) -> &mut Self::Output {
        &mut self.samples_mut()[4]
    }
}

impl<C: Channel> Index<SurroundR> for Frame<C, 6> {
    type Output = C;

    fn index(&self, _: SurroundR) -> &Self::Output {
        &self.samples()[5]
    }
}

impl<C: Channel> IndexMut<SurroundR> for Frame<C, 6> {
    fn index_mut(&mut self, _: SurroundR) -> &mut Self::Output {
        &mut self.samples_mut()[5]
    }
}

////////////////////////////////////////////////////////////

impl<C: Channel> Index<FrontL> for Frame<C, 7> {
    type Output = C;

    fn index(&self, _: FrontL) -> &Self::Output {
        &self.samples()[0]
    }
}

impl<C: Channel> IndexMut<FrontL> for Frame<C, 7> {
    fn index_mut(&mut self, _: FrontL) -> &mut Self::Output {
        &mut self.samples_mut()[0]
    }
}

impl<C: Channel> Index<FrontR> for Frame<C, 7> {
    type Output = C;

    fn index(&self, _: FrontR) -> &Self::Output {
        &self.samples()[1]
    }
}

impl<C: Channel> IndexMut<FrontR> for Frame<C, 7> {
    fn index_mut(&mut self, _: FrontR) -> &mut Self::Output {
        &mut self.samples_mut()[1]
    }
}

impl<C: Channel> Index<Front> for Frame<C, 7> {
    type Output = C;

    fn index(&self, _: Front) -> &Self::Output {
        &self.samples()[2]
    }
}

impl<C: Channel> IndexMut<Front> for Frame<C, 7> {
    fn index_mut(&mut self, _: Front) -> &mut Self::Output {
        &mut self.samples_mut()[2]
    }
}

impl<C: Channel> Index<Lfe> for Frame<C, 7> {
    type Output = C;

    fn index(&self, _: Lfe) -> &Self::Output {
        &self.samples()[3]
    }
}

impl<C: Channel> IndexMut<Lfe> for Frame<C, 7> {
    fn index_mut(&mut self, _: Lfe) -> &mut Self::Output {
        &mut self.samples_mut()[3]
    }
}

impl<C: Channel> Index<Back> for Frame<C, 7> {
    type Output = C;

    fn index(&self, _: Back) -> &Self::Output {
        &self.samples()[4]
    }
}

impl<C: Channel> IndexMut<Back> for Frame<C, 7> {
    fn index_mut(&mut self, _: Back) -> &mut Self::Output {
        &mut self.samples_mut()[4]
    }
}

impl<C: Channel> Index<Left> for Frame<C, 7> {
    type Output = C;

    fn index(&self, _: Left) -> &Self::Output {
        &self.samples()[5]
    }
}

impl<C: Channel> IndexMut<Left> for Frame<C, 7> {
    fn index_mut(&mut self, _: Left) -> &mut Self::Output {
        &mut self.samples_mut()[5]
    }
}

impl<C: Channel> Index<Right> for Frame<C, 7> {
    type Output = C;

    fn index(&self, _: Right) -> &Self::Output {
        &self.samples()[6]
    }
}

impl<C: Channel> IndexMut<Right> for Frame<C, 7> {
    fn index_mut(&mut self, _: Right) -> &mut Self::Output {
        &mut self.samples_mut()[6]
    }
}

////////////////////////////////////////////////////////////

impl<C: Channel> Index<FrontL> for Frame<C, 8> {
    type Output = C;

    fn index(&self, _: FrontL) -> &Self::Output {
        &self.samples()[0]
    }
}

impl<C: Channel> IndexMut<FrontL> for Frame<C, 8> {
    fn index_mut(&mut self, _: FrontL) -> &mut Self::Output {
        &mut self.samples_mut()[0]
    }
}

impl<C: Channel> Index<FrontR> for Frame<C, 8> {
    type Output = C;

    fn index(&self, _: FrontR) -> &Self::Output {
        &self.samples()[1]
    }
}

impl<C: Channel> IndexMut<FrontR> for Frame<C, 8> {
    fn index_mut(&mut self, _: FrontR) -> &mut Self::Output {
        &mut self.samples_mut()[1]
    }
}

impl<C: Channel> Index<Front> for Frame<C, 8> {
    type Output = C;

    fn index(&self, _: Front) -> &Self::Output {
        &self.samples()[2]
    }
}

impl<C: Channel> IndexMut<Front> for Frame<C, 8> {
    fn index_mut(&mut self, _: Front) -> &mut Self::Output {
        &mut self.samples_mut()[2]
    }
}

impl<C: Channel> Index<Lfe> for Frame<C, 8> {
    type Output = C;

    fn index(&self, _: Lfe) -> &Self::Output {
        &self.samples()[3]
    }
}

impl<C: Channel> IndexMut<Lfe> for Frame<C, 8> {
    fn index_mut(&mut self, _: Lfe) -> &mut Self::Output {
        &mut self.samples_mut()[3]
    }
}

impl<C: Channel> Index<BackL> for Frame<C, 8> {
    type Output = C;

    fn index(&self, _: BackL) -> &Self::Output {
        &self.samples()[4]
    }
}

impl<C: Channel> IndexMut<BackL> for Frame<C, 8> {
    fn index_mut(&mut self, _: BackL) -> &mut Self::Output {
        &mut self.samples_mut()[4]
    }
}

impl<C: Channel> Index<BackR> for Frame<C, 8> {
    type Output = C;

    fn index(&self, _: BackR) -> &Self::Output {
        &self.samples()[5]
    }
}

impl<C: Channel> IndexMut<BackR> for Frame<C, 8> {
    fn index_mut(&mut self, _: BackR) -> &mut Self::Output {
        &mut self.samples_mut()[5]
    }
}

impl<C: Channel> Index<Left> for Frame<C, 8> {
    type Output = C;

    fn index(&self, _: Left) -> &Self::Output {
        &self.samples()[6]
    }
}

impl<C: Channel> IndexMut<Left> for Frame<C, 8> {
    fn index_mut(&mut self, _: Left) -> &mut Self::Output {
        &mut self.samples_mut()[6]
    }
}

impl<C: Channel> Index<Right> for Frame<C, 8> {
    type Output = C;

    fn index(&self, _: Right) -> &Self::Output {
        &self.samples()[7]
    }
}

impl<C: Channel> IndexMut<Right> for Frame<C, 8> {
    fn index_mut(&mut self, _: Right) -> &mut Self::Output {
        &mut self.samples_mut()[7]
    }
}
