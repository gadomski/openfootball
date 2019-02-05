#[macro_use]
extern crate failure;

use failure::Error;
use std::path::Path;
use std::str::FromStr;

/// A season of football data.
///
/// Data are provided by https://github.com/openfootball/eng-england.
#[derive(Debug)]
pub struct Season {
    matchdays: Vec<Matchday>,
    name: String,
}

/// A matchday.
#[derive(Debug)]
pub struct Matchday {}

/// A parse error.
#[derive(Debug, Fail)]
pub enum ParseError {
    #[fail(display = "invalid season header: {}", _0)]
    SeasonHeader(String),
}

impl Season {
    /// Creates a new season from the provided path.
    ///
    /// # Examples
    ///
    /// ```
    /// let season = openfootball::Season::from_path("../1-premierleague.txt").unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Season, Error> {
        use std::fs::File;
        use std::io::Read;

        let mut string = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut string)?;
        Ok(string.parse()?)
    }

    /// Returns this season's name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl FromStr for Season {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Season, ParseError> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premier_league() {
        let season = Season::from_path("data/eng-england/2018-19/1-premierleague.txt").unwrap();
        assert_eq!("English Premier League 2018/19", season.name());
    }
}
