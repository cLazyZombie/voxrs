#![allow(clippy::clippy::too_many_arguments)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::clippy::len_without_is_empty)]

mod clock;
pub use clock::Clock;

pub mod io;

mod safecloner;
pub use safecloner::SafeCloner;
