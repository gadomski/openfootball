use super::Stats;
use chrono::NaiveDate;
use std::collections::HashMap;

/// A football game.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Game {
    date: NaiveDate,
    matchday: u16,
    home: String,
    home_score: Option<u16>,
    away: String,
    away_score: Option<u16>,
}

impl Game {
    /// Creates a new game.
    ///
    /// # Examples
    ///
    /// ```
    /// use openfootball::Game;
    /// let game = Game::new(1, "2018-09-01".parse().unwrap(), "THFC", "LFC");
    /// ```
    pub fn new(matchday: u16, date: NaiveDate, home: &str, away: &str) -> Game {
        Game {
            matchday: matchday,
            date: date,
            home: home.to_string(),
            home_score: None,
            away: away.to_string(),
            away_score: None,
        }
    }

    /// Sets this game's score.
    ///
    /// # Examples
    ///
    /// ```
    /// use openfootball::Game;
    /// let game = Game::new(1, "2018-09-01".parse().unwrap(), "THFC", "LFC")
    ///     .with_score(2, 1);
    /// ```
    pub fn with_score(mut self, home: u16, away: u16) -> Game {
        self.home_score = Some(home);
        self.away_score = Some(away);
        self
    }

    /// Returns the home team.
    pub fn home(&self) -> &str {
        &self.home
    }

    /// Returns the away team.
    pub fn away(&self) -> &str {
        &self.away
    }

    /// Updates the stats.
    pub fn update_stats(&self, stats: &mut HashMap<String, Stats>) {
        unimplemented!()
    }
}
