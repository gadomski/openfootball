use chrono::NaiveDate;

/// A football game.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Game {
    date: NaiveDate,
    matchday: u16,
    home: String,
    away: String,
    scores: Option<(u16, u16)>,
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
            away: away.to_string(),
            scores: None,
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
        self.scores = Some((home, away));
        self
    }
}
