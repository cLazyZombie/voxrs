#![warn(clippy::pedantic)]
#![allow(clippy::clippy::too_many_arguments)]
#![allow(clippy::float_cmp)]
#![allow(clippy::new_without_default)]

#[macro_use]
extern crate voxrs_derive;

pub mod asset;
pub mod blueprint;
//pub mod camera;
pub mod ecs;
pub mod io;
pub mod render;
pub mod safecloner;
pub mod texture;
