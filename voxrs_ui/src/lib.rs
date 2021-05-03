#![allow(dead_code)]
//mod widget;
//pub use widget::*;

mod comp;

pub mod system;

mod res;
pub use res::init_resources;
pub use res::InputQueue;

mod widget;
pub use widget::*;

pub mod input;

pub mod output;

mod widget_builder;
pub use widget_builder::WidgetBuilder;
