use winit::dpi::PhysicalSize;

use crate::blueprint::Blueprint;

pub enum Command {
    Render(Blueprint),
    Resize(PhysicalSize<u32>),
    Exit,
}
