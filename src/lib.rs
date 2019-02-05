#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate regex;

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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Matchday {
    number: u16,
    games: Vec<Game>,
}

/// A game.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
        let mut season: Season = string.parse()?;
        season.matchdays.sort();
        Ok(season)
    }

    /// Returns this season's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns this season's teams.
    pub fn teams(&self) -> Vec<String> {
        use std::collections::HashSet;
        let mut teams = HashSet::new();
        for matchday in &self.matchdays {
            for game in &matchday.games {
                teams.insert(game.home_team.to_string());
                teams.insert(game.away_team.to_string());
            }
        }
        let mut teams: Vec<String> = teams.drain().collect();
        teams.sort();
        teams
    }

    /// Returns elo ratings for each match day.
    pub fn ratings(&self) -> BTreeMap<u16, BTreeMap<String, i32>> {
        let mut matchday_ratings: BTreeMap<u16, BTreeMap<String, i32>> = BTreeMap::new();
        let mut ratings: BTreeMap<String, i32> =
            self.teams().into_iter().map(|team| (team, 1500)).collect();
        let expected_score = |a, b| 1. / (1. + 10f64.powf((b - a) / 400.));
        let update = |score: f64, expected: f64| (32. * (score - expected)).round() as i32;
        for matchday in &self.matchdays {
            let mut has_unplayed_games = false;
            for game in &matchday.games {
                let home_rating = f64::from(ratings[&game.home_team]);
                let away_rating = f64::from(ratings[&game.away_team]);
                let home_expected = expected_score(home_rating, away_rating);
                let away_expected = expected_score(home_rating, away_rating);
                let (home_score, away_score) = if let Some((home_score, away_score)) = game
                    .home_score
                    .and_then(|home| game.away_score.map(|away| (home, away)))
                {
                    if home_score > away_score {
                        (1., 0.)
                    } else if away_score > home_score {
                        (0., 1.)
                    } else {
                        (0.5, 0.5)
                    }
                } else {
                    has_unplayed_games = true;
                    continue;
                };
                *ratings.get_mut(&game.home_team).unwrap() += update(home_score, home_expected);
                *ratings.get_mut(&game.away_team).unwrap() += update(away_score, away_expected);
            }
            if !has_unplayed_games {
                matchday_ratings.insert(matchday.number, ratings.clone());
            }
        }
        matchday_ratings
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
            } else if date_regex.is_match(line) {
                // pass
            } else {
                matchday.add_game(line.parse()?);
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
            games: Vec::new(),
        }
    }

    /// Adds a game to this matchday.
    ///
    /// # Examples
    ///
    /// ```
    /// use openfootball::Matchday;
    /// let mut matchday = Matchday::new(20);
    /// matchday.add_game("  Everton FC 2-6 Tottenham Hotspur".parse().unwrap());
    /// ```
    pub fn add_game(&mut self, game: Game) {
        self.games.push(game)
    }

    /// Returns true if this matchday is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut matchday = openfootball::Matchday::new(20);
    /// assert!(matchday.is_empty());
    /// matchday.add_game("  Everton FC 2-6 Tottenham Hotspur".parse().unwrap());
    /// assert!(!matchday.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.games.is_empty()
    }
}

impl FromStr for Game {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Game, ParseError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
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
        }
        if let Some(captures) = RE.captures(s) {
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
    }
}
