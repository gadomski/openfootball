#[macro_use]
extern crate failure;

use failure::Error;
use std::fs::File;
use std::io::Lines;
use std::path::Path;

/// A season of football data.
///
/// Data are provided by https://github.com/openfootball/eng-england.
pub struct Season {
    matchdays: Vec<Matchday>,
    name: String,
}

/// A matchday.
pub struct Matchday {}

/// An error raised by an unexpected end of file when parsing input data.
#[derive(Debug, Fail)]
#[fail(display = "unexpected end of file")]
pub struct UnexpectedEndOfFile;

impl Season {
    /// Creates a new season from the provided path.
    ///
    /// # Examples
    ///
    /// ```
    /// let season = openfootball::Season::from_path("../1-premierleague.txt").unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Season, Error> {}

    /// Returns this season's name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premier_league() {
        let season = Season::from_path("1-premierleague.txt").unwrap();
        assert_eq!("English Premier League 2018/19", season.name());
    }
}
