extern crate clap;
extern crate csv;
extern crate failure;
extern crate openfootball;

use clap::{App, Arg};
use failure::Error;

fn main() -> Result<(), Error> {
    let _matches = App::new("openfootball")
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
    Ok(())
}
