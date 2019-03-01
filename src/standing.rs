use chrono::NaiveDate;

/// A team's standing at the end of a day in a season.
#[derive(Debug, Serialize)]
pub struct Standing {
    team: String,
    date: NaiveDate,
    matchweek: u16,
    #[serde(flatten)]
    stats: Stats,
}

#[derive(Debug, Serialize)]
pub struct Stats {
    wins: u16,
    draws: u16,
    losses: u16,
    goals_for: u16,
    goals_against: u16,
    elo_rating: i32,
}

impl Stats {
    /// Creates a new stats with the provided initial elo rating.
    pub fn new(initial_elo_rating: i32) -> Stats {
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
