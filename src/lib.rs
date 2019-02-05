#[macro_use]
extern crate failure;
extern crate regex;

use chrono::NaiveDate;
use failure::Error;
use regex::Regex;
use std::collections::BTreeMap;
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
pub struct Matchday {
    number: u16,
    games: BTreeMap<NaiveDate, Vec<Game>>,
}

/// A game.
#[derive(Debug)]
pub struct Game {
    home_team: String,
    away_team: String,
    home_score: Option<u16>,
    away_score: Option<u16>,
}

/// A parse error.
#[derive(Debug, Fail)]
pub enum ParseError {
    #[fail(display = "invalid header content: {}", _0)]
    InvalidHeader(String),

    #[fail(display = "invalid game: {}", _0)]
    InvalidGame(String),

    #[fail(display = "the input file ended at an unexpected spot")]
    UnexpectedEndOfFile,
}

impl Season {
    /// Creates a new season from the provided path.
    ///
    /// # Examples
    ///
    /// ```
    /// let season = openfootball::Season::from_path("data/eng-england/2018-19/1-premierleague.txt").unwrap();
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
        let mut lines = s.lines();
        let header = lines.next().ok_or(ParseError::UnexpectedEndOfFile)?;
        if header != "###################################" {
            return Err(ParseError::InvalidHeader(header.to_string()));
        }
        let name: String = lines
            .next()
            .ok_or(ParseError::UnexpectedEndOfFile)?
            .chars()
            .skip(2)
            .collect();

        let mut matchday = Matchday::new(0);
        let mut date = NaiveDate::from_ymd(2018, 8, 10);
        let mut matchdays = Vec::new();
        let matchday_regex = Regex::new(r"^Matchday (?P<matchday>\d+)$").unwrap();
        let date_regex = Regex::new(r"^\[\w+ (?P<date>(?P<month>\w+)/\d+)\]$").unwrap();
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            if let Some(captures) = matchday_regex.captures(line) {
                if !matchday.is_empty() {
                    matchdays.push(matchday);
                }
                matchday = Matchday::new(captures["matchday"].parse().unwrap());
            } else if let Some(captures) = date_regex.captures(line) {
                let year = if ["Aug", "Sep", "Oct", "Nov", "Dec"].contains(&&captures["month"]) {
                    2018
                } else {
                    2019
                };
                date = NaiveDate::parse_from_str(
                    &format!("{} {}", year, &captures["date"]),
                    "%Y %b/%d",
                )
                .unwrap();
            } else {
                matchday.add_game(date, line.parse()?);
            }
        }
        Ok(Season {
            name: name,
            matchdays: matchdays,
        })
    }
}

impl Matchday {
    /// Creates a new matchday with the provided number.
    ///
    /// # Examples
    ///
    /// ```
    /// let matchday = openfootball::Matchday::new(0);
    /// ```
    pub fn new(number: u16) -> Matchday {
        Matchday {
            number: number,
            games: BTreeMap::new(),
        }
    }

    /// Adds a game to this matchday.
    ///
    /// # Examples
    ///
    /// ```
    /// use openfootball::Matchday;
    /// let mut matchday = Matchday::new(20);
    /// matchday.add_game("2018-12-23".parse().unwrap(),
    ///     "  Everton FC 2-6 Tottenham Hotspur".parse().unwrap());
    /// ```
    pub fn add_game(&mut self, date: NaiveDate, game: Game) {
        self.games.entry(date).or_insert_with(Vec::new).push(game)
    }

    /// Returns true if this matchday is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let matchday = openfootball::Matchday::new(20);
    /// assert!(matchday.is_empty());
    /// matchday.add_game("2018-12-23".parse().unwrap(),
    ///     "  Everton FC 2-6 Tottenham Hotspur".parse().unwrap());
    /// assert!(!matchday.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.games.is_empty()
    }
}

impl FromStr for Game {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Game, ParseError> {
        let regex = Regex::new(
            r"(?x)^
            \s+
            (?P<home_team>.*?)
            \s+
            (?P<home_score>\d+)?
            -
            (?P<away_score>\d+)?
            \s
            (?P<away_team>.*)
            $
            ",
        )
        .unwrap();
        if let Some(captures) = regex.captures(s) {
            Ok(Game {
                home_team: captures["home_team"].to_string(),
                home_score: captures
                    .name("home_score")
                    .map(|m| m.as_str().parse().unwrap()),
                away_team: captures["away_team"].to_string(),
                away_score: captures
                    .name("away_score")
                    .map(|m| m.as_str().parse().unwrap()),
            })
        } else {
            Err(ParseError::InvalidGame(s.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premier_league() {
        let season = Season::from_path("data/eng-england/2018-19/1-premierleague.txt").unwrap();
        assert_eq!("English Premier League 2018/19", season.name());
        assert!(false);
    }
}
