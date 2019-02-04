extern crate failure;

use failure::Error;
use std::path::Path;

/// A season of the English Premier League (EPL).
///
/// Data are provided by https://github.com/openfootball/eng-england.
pub struct Season {}

impl Season {
    /// Creates a new season from the provided path.
    ///
    /// # Examples
    ///
    /// ```
    /// let season = openfootball::Season::from_path("../1-premierleague.txt").unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Season, Error> {
        Ok(Season {})
    }
}
