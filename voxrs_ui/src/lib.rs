#![allow(dead_code)]
//mod widget;
//pub use widget::*;

mod comp;

pub mod system;

mod res;
pub use res::init_resources;
pub use res::InputQueue;
pub use res::OutputQueue;
pub use res::ScreenResolution;

mod widget;
pub use widget::*;

pub mod input;

mod interaction;
pub use interaction::Interaction;

mod widget_builder;
pub use widget_builder::WidgetBuilder;

mod anchor;
pub use anchor::AnchorHorizon;
pub use anchor::AnchorVertical;
