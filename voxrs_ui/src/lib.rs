#![allow(dead_code)]
//mod widget;
//pub use widget::*;

mod comp;

pub mod system;

mod res;
pub use res::InputQueue;

mod widget;
pub use widget::*;

pub mod input;

mod widget_repository;
pub use widget_repository::WidgetRepository;

mod widget_builder;
pub use widget_builder::WidgetBuilder;
