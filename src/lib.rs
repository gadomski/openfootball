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

const INITIAL_ELO_RATING: i16 = 1500;
const K: f64 = 32.;

/// A season of football data.
///
/// Data are provided by https://github.com/openfootball/eng-england.
#[derive(Debug)]
pub struct Season {
    matchdays: Vec<Matchday>,
    name: String,
}

/// A matchday.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Matchday {
    number: u16,
    games: Vec<Game>,
}

/// A game.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Game {
    home: String,
    away: String,
    scores: Option<Scores>,
}

/// A game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Scores {
    home: u16,
    away: u16,
}

/// The league standings at a point in time.
pub type Standings = BTreeMap<String, Position>;

/// The position of a team at a point in time.
#[derive(Clone, Debug, Default)]
pub struct Position {
    matches_played: u16,
    wins: u16,
    draws: u16,
    losses: u16,
    goals_for: u16,
    goals_against: u16,
    goal_differential: i32,
    points: u16,
    elo_rating: i16,
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

    /// Returns this season's teams in sorted order.
    pub fn teams(&self) -> Vec<String> {
        use std::collections::HashSet;
        let mut teams = HashSet::new();
        for matchday in &self.matchdays {
            for game in &matchday.games {
                teams.insert(game.home.to_string());
                teams.insert(game.away.to_string());
            }
        }
        let mut teams: Vec<String> = teams.drain().collect();
        teams.sort();
        teams
    }

    /// Returns standings after each match day.
    pub fn standings(&self) -> BTreeMap<Matchday, Standings> {
        let mut matchday_standings: BTreeMap<Matchday, Standings> = BTreeMap::new();
        let mut standings = self.initial_standings();
        for matchday in &self.matchdays {
            for game in matchday.games.iter().filter(|game| game.is_played()) {
                let mut home = standings.remove(&game.home).unwrap();
                let mut away = standings.remove(&game.away).unwrap();
                game.update_positions(&mut home, &mut away);
                standings.insert(game.home.clone(), home);
                standings.insert(game.away.clone(), away);
            }
            matchday_standings.insert(matchday.clone(), standings.clone());
        }
        matchday_standings
    }

    fn initial_standings(&self) -> Standings {
        self.teams()
            .into_iter()
            .map(|team| (team, Position::new()))
            .collect()
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

impl Game {
    fn is_played(&self) -> bool {
        self.scores.is_some()
    }

    fn update_positions(&self, home: &mut Position, away: &mut Position) {
        let scores = if let Some(scores) = &self.scores {
            scores
        } else {
            return;
        };

        home.matches_played += 1;
        home.goals_for += scores.home;
        home.goals_against += scores.away;
        home.goal_differential += i32::from(scores.home) - i32::from(scores.away);
        away.matches_played += 1;
        away.goals_for += scores.away;
        away.goals_against += scores.home;
        away.goal_differential += i32::from(scores.away) - i32::from(scores.home);

        let (home_score, away_score) = if scores.home > scores.away {
            home.points += 3;
            (1., 0.)
        } else if scores.home < scores.away {
            away.points += 1;
            (0., 1.)
        } else {
            home.points += 1;
            away.points += 1;
            (0.5, 0.5)
        };

        let expected = |a: &Position, b: &Position| {
            1. / (1. + 10f64.powf((f64::from(b.elo_rating) - f64::from(a.elo_rating)) / 400.))
        };
        home.elo_rating += (K * (home_score - expected(home, away))).round() as i16;
        away.elo_rating += (K * (away_score - expected(away, home))).round() as i16;
    }
}

impl FromStr for Game {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Game, ParseError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)^
            \s+
            (?P<home>.*?)
            \s+
            (?P<home_score>\d+)?
            -
            (?P<away_score>\d+)?
            \s
            (?P<away>.*)
            $
            ",
            )
            .unwrap();
        }
        if let Some(captures) = RE.captures(s) {
            let home = captures["home"].to_string();
            let away = captures["away"].to_string();
            let scores = if let Some((home_score, away_score)) =
                captures.name("home_score").and_then(|home_score| {
                    captures
                        .name("away_score")
                        .map(|away_score| (home_score, away_score))
                }) {
                Some(Scores {
                    home: home_score.as_str().parse().unwrap(),
                    away: away_score.as_str().parse().unwrap(),
                })
            } else {
                None
            };
            Ok(Game {
                home: home,
                away: away,
                scores: scores,
            })
        } else {
            Err(ParseError::InvalidGame(s.to_string()))
        }
    }
}

impl Position {
    fn new() -> Position {
        Position {
            elo_rating: INITIAL_ELO_RATING,
            ..Default::default()
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
