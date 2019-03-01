//! A library to read opendata text files.

extern crate chrono;
#[macro_use]
extern crate failure;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use chrono::NaiveDate;
use std::collections::HashMap;
use std::path::Path;

/// A season of football data.
///
/// This was set up to work with openfootball's 2018-2019 English Premier league data, available
/// from https://github.com/openfootball/eng-england. Your mileage may vary if you try to use with
/// other data.
#[derive(Debug)]
pub struct Season {
    games: Vec<Game>,
}

/// A football game.
#[derive(Debug)]
pub struct Game {
    date: NaiveDate,
    matchweek: u16,
    home: String,
    away: String,
    scores: Option<Scores>,
}

#[derive(Debug)]
struct Scores {
    home: u16,
    away: u16,
}

/// A team's standing at the end of a day.
#[derive(Debug, Serialize)]
pub struct Standing {
    team: String,
    date: NaiveDate,
    matchweek: u16,
    wins: u16,
    draws: u16,
    losses: u16,
    goals_for: u16,
    goals_against: u16,
    elo_rating: i32,
}

/// A team's accumulated statistics throughout the season.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Stats {
    wins: u16,
    draws: u16,
    losses: u16,
    goals_for: u16,
    goals_against: u16,
    elo_rating: i32,
}

/// Crate-specific errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// This line of the season file could not be parsed.
    #[fail(display = "invalid season line: {}", _0)]
    InvalidSeasonLine(String),

    /// This team was missing from the stats map when trying to calculate standings.
    #[fail(display = "missing team: {}", _0)]
    MissingTeam(String),
}

impl Season {
    /// Reads a season from a path on the filesystem.
    ///
    /// # Examples
    ///
    /// ```
    /// use openfootball::Season;
    /// let season = Season::from_path("tests/data/pl.txt").unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Season, failure::Error> {
        use chrono::{Datelike, NaiveDate, Utc};
        use regex::Regex;
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let mut year = 0i32;
        let header_regex = Regex::new(r"^# (?P<name>.+) (?P<year>\d{4})/\d{2}$").unwrap();
        let mut matchweek = 0u16;
        let matchday_regex = Regex::new(r"^Matchday (?P<matchweek>\d+)$").unwrap();
        let mut date = Utc::today().naive_utc();
        let date_regex =
            Regex::new(r"^\[[[:alpha:]]{3} (?P<month>[[:alpha:]]{3})/(?P<day>\d+)\]$").unwrap();
        let game_regex = Regex::new(
            r"(?x)
            ^
            (?P<home>.+?)
            \s+
            (?P<home_score>\d+)?
            -
            (?P<away_score>\d+)?
            \s+
            (?P<away>.+?)
            (\s*postponed)?
            $
        ",
        )
        .unwrap();

        let mut games = Vec::new();
        for result in BufReader::new(File::open(path)?).lines() {
            let line = result?;
            let line = line.trim();
            if line.is_empty() || line.chars().all(|c| c == '#') {
                continue;
            } else if let Some(captures) = header_regex.captures(line) {
                year = captures.name("year").unwrap().as_str().parse()?;
            } else if let Some(captures) = matchday_regex.captures(line) {
                matchweek = captures.name("matchweek").unwrap().as_str().parse()?;
            } else if let Some(captures) = date_regex.captures(line) {
                date = NaiveDate::parse_from_str(
                    &format!(
                        "{} {} {}",
                        year,
                        captures.name("month").unwrap().as_str(),
                        captures.name("day").unwrap().as_str()
                    ),
                    "%Y %b %d",
                )?;
                if date.month() < 7 {
                    date = date.with_year(year + 1).unwrap();
                }
            } else if let Some(captures) = game_regex.captures(line) {
                if let Some(home_score) = captures.name("home_score") {
                    let home_score = home_score.as_str().parse::<u16>()?;
                    if let Some(away_score) = captures.name("away_score") {
                        let away_score = away_score.as_str().parse::<u16>()?;
                        let home = captures.name("home").unwrap().as_str();
                        let away = captures.name("away").unwrap().as_str();
                        games.push(
                            Game::new(matchweek, date, home, away)
                                .with_scores(home_score, away_score),
                        );
                    }
                }
            } else {
                return Err(Error::InvalidSeasonLine(line.to_string()).into());
            }
        }

        Ok(Season { games: games })
    }

    /// Returns this season's played games as a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use openfootball::Season;
    /// let season = Season::from_path("tests/data/pl.txt").unwrap();
    /// let games = season.games();
    /// ```
    pub fn games(&self) -> &[Game] {
        &self.games
    }

    /// Returns this season's standings.
    ///
    /// These are calculated from all played games.
    ///
    /// # Examples
    ///
    /// ```
    /// use openfootball::Season;
    /// let season = Season::from_path("tests/data/pl.txt").unwrap();
    /// let standings = season.standings(1500, 32.);
    /// ```
    pub fn standings(&self, initial_elo_rating: i32, k: f64) -> Result<Vec<Standing>, Error> {
        use std::collections::{HashMap, HashSet};

        let mut teams = HashSet::new();
        for game in &self.games {
            teams.insert(game.home().to_string());
            teams.insert(game.away().to_string());
        }
        let mut stats: HashMap<_, _> = teams
            .iter()
            .map(|team| (team.to_string(), Stats::new(initial_elo_rating)))
            .collect();
        let mut standings = Vec::new();
        for game in &self.games {
            if let Some((home, away)) = game.update_stats(&mut stats, k)? {
                standings.push(home);
                standings.push(away);
            }
        }
        Ok(standings)
    }
}

impl Game {
    /// Creates a new game for a given matdhay, date, home, and away teams.
    ///
    /// # Examples
    ///
    /// ```
    /// use openfootball::Game;
    /// let game = Game::new(1, "2018-08-11".parse().unwrap(), "Newcastle United", "Tottenham Hotspur");
    /// ```
    pub fn new(matchweek: u16, date: NaiveDate, home: &str, away: &str) -> Game {
        Game {
            matchweek: matchweek,
            date: date,
            home: home.to_string(),
            away: away.to_string(),
            scores: None,
        }
    }

    /// Sets the scores for a game.
    ///
    /// # Examples
    ///
    /// ```
    /// use openfootball::Game;
    /// let game = Game::new(1, "2018-08-11".parse().unwrap(), "Newcastle United", "Tottenham Hotspur")
    ///     .with_scores(1, 2);
    /// ```
    pub fn with_scores(mut self, home: u16, away: u16) -> Game {
        self.scores = Some(Scores {
            home: home,
            away: away,
        });
        self
    }

    /// Returns the home team's name.
    ///
    /// # Examples
    ///
    /// ```
    /// use openfootball::Game;
    /// let game = Game::new(1, "2018-08-11".parse().unwrap(), "Newcastle United", "Tottenham Hotspur");
    /// assert_eq!("Newcastle United", game.home());
    /// ```
    pub fn home(&self) -> &str {
        &self.home
    }

    /// Returns the away team's name.
    ///
    /// # Examples
    ///
    /// ```
    /// use openfootball::Game;
    /// let game = Game::new(1, "2018-08-11".parse().unwrap(), "Newcastle United", "Tottenham Hotspur");
    /// assert_eq!("Tottenham Hotspur", game.away());
    /// ```
    pub fn away(&self) -> &str {
        &self.away
    }

    fn update_stats(
        &self,
        stats: &mut HashMap<String, Stats>,
        k: f64,
    ) -> Result<Option<(Standing, Standing)>, Error> {
        let scores = if let Some(scores) = &self.scores {
            scores
        } else {
            return Ok(None);
        };
        let home_rating = f64::from(
            stats
                .get(&self.home)
                .ok_or(Error::MissingTeam(self.home.to_string()))?
                .elo_rating,
        );
        let away_rating = f64::from(
            stats
                .get(&self.away)
                .ok_or(Error::MissingTeam(self.away.to_string()))?
                .elo_rating,
        );
        let mut update = |team: &str,
                          goals_for: u16,
                          goals_against: u16,
                          expected: f64|
         -> Result<Standing, Error> {
            let stats = stats
                .get_mut(team)
                .ok_or(Error::MissingTeam(team.to_string()))?;
            stats.goals_for += goals_for;
            stats.goals_against += goals_against;
            let actual = if goals_for > goals_against {
                stats.wins += 1;
                1.
            } else if goals_for < goals_against {
                stats.losses += 1;
                0.
            } else {
                stats.draws += 1;
                0.5
            };
            stats.elo_rating += (k * (actual - expected)).round() as i32;

            Ok(Standing {
                matchweek: self.matchweek,
                date: self.date,
                team: team.to_string(),
                wins: stats.wins,
                draws: stats.draws,
                losses: stats.losses,
                goals_for: stats.goals_for,
                goals_against: stats.goals_against,
                elo_rating: stats.elo_rating,
            })
        };
        let expected_home = 1. / (1. + 10f64.powf((away_rating - home_rating) / 400.));
        let expected_away = 1. - expected_home;
        let home = update(&self.home, scores.home, scores.away, expected_home)?;
        let away = update(&self.away, scores.away, scores.home, expected_away)?;
        Ok(Some((home, away)))
    }
}

impl Stats {
    fn new(initial_elo_rating: i32) -> Stats {
        Stats {
            wins: 0,
            draws: 0,
            losses: 0,
            goals_for: 0,
            goals_against: 0,
            elo_rating: initial_elo_rating,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn season_game() {
        let season = Season::from_path("tests/data/pl.txt").unwrap();
        assert_eq!(279, season.games().len());
    }
}
