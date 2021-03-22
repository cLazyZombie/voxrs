use enumflags2::bitflags;

/// XPos : X Positive direction , XNeg : X Negative direction
#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Dir {
    XPos = 0b0000_0001,
    XNeg = 0b0000_0010,
    YPos = 0b0000_0100,
    YNeg = 0b0000_1000,
    ZPos = 0b0001_0000,
    ZNeg = 0b0010_0000,
}

impl Dir {
    pub fn opposite_dir(&self) -> Self {
        match *self {
            Dir::XPos => Dir::XNeg,
            Dir::XNeg => Dir::XPos,
            Dir::YPos => Dir::YNeg,
            Dir::YNeg => Dir::YPos,
            Dir::ZPos => Dir::ZNeg,
            Dir::ZNeg => Dir::ZPos,
        }
    }
}
