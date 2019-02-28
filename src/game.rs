/// A football game.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Game {
    home: String,
    away: String,
    scores: Option<Scores>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct Scores {
    home: u16,
    away: u16,
}
