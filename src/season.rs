use super::Game;
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
        let mut matchday = 0u16;
        let matchday_regex = Regex::new(r"^Matchday (?P<matchday>\d+)$").unwrap();
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
            if line.is_empty() {
                continue;
            } else if let Some(captures) = header_regex.captures(line) {
                year = captures.name("year").unwrap().as_str().parse()?;
            } else if let Some(captures) = matchday_regex.captures(line) {
                matchday = captures.name("matchday").unwrap().as_str().parse()?;
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
                            Game::new(matchday, date, home, away)
                                .with_score(home_score, away_score),
                        );
                    }
                }
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
}
