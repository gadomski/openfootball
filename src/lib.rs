//! A library to read opendata text files.

extern crate chrono;
extern crate failure;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod game;
mod season;
mod standing;

pub use game::Game;
pub use season::Season;
pub use standing::{Standing, Stats};
