//! A library to read opendata text files.

#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod game;
mod season;

pub use game::Game;
pub use season::Season;
