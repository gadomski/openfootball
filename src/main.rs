extern crate clap;
extern crate failure;
extern crate openfootball;
extern crate serde;
extern crate serde_json;

use clap::{App, Arg};
use failure::Error;
use openfootball::Season;

fn main() -> Result<(), Error> {
    let matches = App::new("openfootball")
        .arg(
            Arg::with_name("INFILE")
                .help("Sets the input openfootball text file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("score-factor")
                .short("s")
                .long("score-factor")
                .takes_value(true)
                .help("Sets the score factor, the amount the score influences the Elo rating"),
        )
        .arg(
            Arg::with_name("k")
                .short("k")
                .takes_value(true)
                .help("Sets the k value, the sensitivity of the Elo rating"),
        )
        .get_matches();
    let infile = matches.value_of("INFILE").unwrap();
    let mut season = Season::from_path(infile)?;
    if let Some(score_factor) = matches.value_of("score-factor") {
        season.set_score_factor(score_factor.parse()?);
    }
    if let Some(k) = matches.value_of("k") {
        season.set_k(k.parse()?);
    }
    let standings = season.standings();
    println!("{}", serde_json::to_string_pretty(&standings)?);
    Ok(())
}
